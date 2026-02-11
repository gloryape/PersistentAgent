#!/usr/bin/env python3
"""
Optic Nerve - Transducer for external video/desktop capture.

Captures raw RGB and audio and streams them as SENS protocol to stdout
for the Entity to ingest.

SENS Protocol:
[HEADER: 4 bytes "SENS"]
[TYPE:   1 byte (1=Video, 2=Audio)]
[SIZE:   4 bytes u32 LE]
[PAYLOAD: size bytes]

One frame = Video chunk (12288 bytes) + Audio chunk (8192 bytes)

Output rate: 90 Hz to match the Entity's metabolic clock.
Video frames are repeated as needed to maintain 90 Hz regardless of
the source video's native frame rate.
"""

import sys
import struct
import time
import argparse
import subprocess
import tempfile
import os
from pathlib import Path

try:
    import cv2
    import numpy as np
except ImportError as e:
    print(f"ERROR: Missing dependency: {e}", file=sys.stderr)
    print("Install with: pip install opencv-python-headless numpy", file=sys.stderr)
    sys.exit(1)

# Constants
COGNITIVE_RESOLUTION = (64, 64)  # Entity's visual grid
AUDIO_SAMPLES_PER_FRAME = 2048
AUDIO_SAMPLE_RATE = 44100

# Match Entity's 90 Hz metabolic clock
TARGET_FPS = 90
FRAME_INTERVAL = 1.0 / TARGET_FPS

# SENS Protocol constants
SENS_HEADER = b"SENS"
TYPE_VIDEO = 1
TYPE_AUDIO = 2
VIDEO_PAYLOAD_SIZE = 64 * 64 * 3  # 12288 bytes
AUDIO_PAYLOAD_SIZE = 2048 * 4     # 8192 bytes


def rgb_to_bytes(frame_rgb):
    """Convert 64x64 RGB numpy array to bytes."""
    if frame_rgb.dtype != np.uint8:
        frame_rgb = (frame_rgb.clip(0, 255)).astype(np.uint8)
    return frame_rgb.tobytes()


def write_sens_chunk(chunk_type: int, payload: bytes):
    """Write a single SENS chunk: header + type + size + payload."""
    sys.stdout.buffer.write(SENS_HEADER)
    sys.stdout.buffer.write(bytes([chunk_type]))
    size = len(payload)
    sys.stdout.buffer.write(struct.pack("<I", size))  # u32 LE
    sys.stdout.buffer.write(payload)
    sys.stdout.buffer.flush()


def write_frame(rgb_grid, audio_samples):
    """
    Write RGB grid and audio samples as SENS protocol chunks.

    Args:
        rgb_grid: numpy array of shape (64, 64, 3) with dtype uint8
        audio_samples: numpy array of shape (2048,) with dtype float32
    """
    # Ensure audio is exactly 2048 samples, float32
    if len(audio_samples) != AUDIO_SAMPLES_PER_FRAME:
        if len(audio_samples) < AUDIO_SAMPLES_PER_FRAME:
            audio_samples = np.pad(
                audio_samples,
                (0, AUDIO_SAMPLES_PER_FRAME - len(audio_samples)),
                mode='constant'
            )
        else:
            audio_samples = audio_samples[:AUDIO_SAMPLES_PER_FRAME]

    # Write Video chunk (type 1)
    rgb_bytes = rgb_to_bytes(rgb_grid)
    write_sens_chunk(TYPE_VIDEO, rgb_bytes)

    # Write Audio chunk (type 2)
    audio_f32 = audio_samples.astype(np.float32)
    audio_bytes = audio_f32.tobytes()
    write_sens_chunk(TYPE_AUDIO, audio_bytes)


def find_ffmpeg():
    """Find ffmpeg executable, checking common locations."""
    # Try PATH first
    import shutil
    ffmpeg_path = shutil.which("ffmpeg")
    if ffmpeg_path:
        return ffmpeg_path
    
    # Check common Windows install locations
    common_paths = [
        r"C:\ffmpeg-2026-02-09-git-9bfa1635ae-essentials_build\bin\ffmpeg.exe",
        r"C:\ffmpeg\bin\ffmpeg.exe",
        r"C:\Program Files\ffmpeg\bin\ffmpeg.exe",
    ]
    
    for path in common_paths:
        if os.path.exists(path):
            return path
    
    return None


def extract_audio_track(video_path):
    """
    Try to extract raw PCM audio from video file using ffmpeg.
    Returns numpy float32 array of all audio samples, or None if ffmpeg unavailable.
    """
    ffmpeg_exe = find_ffmpeg()
    if not ffmpeg_exe:
        print("ffmpeg not found - video audio will be silent", file=sys.stderr)
        print("  Install ffmpeg and add to PATH, or place in C:\\ffmpeg\\bin\\", file=sys.stderr)
        return None
    
    try:
        result = subprocess.run(
            [
                ffmpeg_exe, "-i", str(video_path),
                "-f", "f32le",          # raw float32 little-endian
                "-acodec", "pcm_f32le",
                "-ac", "1",             # mono
                "-ar", str(AUDIO_SAMPLE_RATE),
                "-v", "quiet",
                "pipe:1"
            ],
            capture_output=True,
            timeout=120
        )
        if result.returncode == 0 and len(result.stdout) > 0:
            audio = np.frombuffer(result.stdout, dtype=np.float32)
            print(f"Extracted {len(audio)} audio samples ({len(audio)/AUDIO_SAMPLE_RATE:.1f}s)", file=sys.stderr)
            return audio
        else:
            print("ffmpeg returned no audio data", file=sys.stderr)
            if result.stderr:
                print(f"  ffmpeg stderr: {result.stderr.decode('utf-8', errors='ignore')[:200]}", file=sys.stderr)
            return None
    except Exception as e:
        print(f"Audio extraction failed: {e}", file=sys.stderr)
        return None


class VideoSource:
    """
    Reads video frames from a file, loops on end, and provides
    frames at 90 Hz by repeating as needed.
    """

    def __init__(self, video_path: str, loop: bool = True):
        self.video_path = video_path
        self.loop = loop
        self.cap = cv2.VideoCapture(str(video_path))
        if not self.cap.isOpened():
            raise RuntimeError(f"Could not open video file: {video_path}")

        self.native_fps = self.cap.get(cv2.CAP_PROP_FPS) or 30.0
        self.total_frames = int(self.cap.get(cv2.CAP_PROP_FRAME_COUNT))
        self.frame_interval = 1.0 / self.native_fps

        # How many 90Hz ticks per video frame
        self.ticks_per_frame = max(1, round(TARGET_FPS / self.native_fps))

        self._current_frame = None  # cached RGB 64x64
        self._tick_counter = 0      # counts 90Hz ticks since last frame read
        self._frames_read = 0
        self._loops = 0

        # Audio track (pre-extracted if ffmpeg available)
        self.audio_track = extract_audio_track(video_path)
        self._audio_cursor = 0  # position in audio_track

        duration = self.total_frames / self.native_fps if self.native_fps > 0 else 0
        print(f"Video: {Path(video_path).name}", file=sys.stderr)
        print(f"  Resolution: {int(self.cap.get(cv2.CAP_PROP_FRAME_WIDTH))}x{int(self.cap.get(cv2.CAP_PROP_FRAME_HEIGHT))}", file=sys.stderr)
        print(f"  Native FPS: {self.native_fps:.2f}", file=sys.stderr)
        print(f"  Duration: {duration:.1f}s ({self.total_frames} frames)", file=sys.stderr)
        print(f"  Ticks per frame: {self.ticks_per_frame} (output at {TARGET_FPS} Hz)", file=sys.stderr)
        print(f"  Audio: {'extracted' if self.audio_track is not None else 'silent (no ffmpeg)'}", file=sys.stderr)
        print(f"  Loop: {self.loop}", file=sys.stderr)

    def _read_next_video_frame(self):
        """Read next frame from video, loop if needed. Returns False if done."""
        ret, frame = self.cap.read()
        if not ret:
            if self.loop:
                self._loops += 1
                self.cap.set(cv2.CAP_PROP_POS_FRAMES, 0)
                if self.audio_track is not None:
                    self._audio_cursor = 0
                print(f"Video looped (loop #{self._loops})", file=sys.stderr)
                ret, frame = self.cap.read()
                if not ret:
                    return False
            else:
                return False

        # Convert BGR (OpenCV default) to RGB and resize to 64x64
        frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        self._current_frame = cv2.resize(
            frame_rgb, COGNITIVE_RESOLUTION, interpolation=cv2.INTER_LINEAR
        )
        self._frames_read += 1
        return True

    def next_frame(self):
        """
        Get the next 64x64 RGB frame at 90Hz rate.
        Repeats frames as needed to match the organism's clock.
        Returns (rgb_64x64, audio_2048) or (None, None) if done.
        """
        # Do we need a new video frame?
        if self._current_frame is None or self._tick_counter >= self.ticks_per_frame:
            if not self._read_next_video_frame():
                return None, None
            self._tick_counter = 0

        self._tick_counter += 1

        # Audio: slice from pre-extracted track, or silence
        audio = np.zeros(AUDIO_SAMPLES_PER_FRAME, dtype=np.float32)
        if self.audio_track is not None:
            samples_per_tick = AUDIO_SAMPLE_RATE // TARGET_FPS  # ~490 samples per tick
            start = self._audio_cursor
            end = start + AUDIO_SAMPLES_PER_FRAME
            if end <= len(self.audio_track):
                audio = self.audio_track[start:end].copy()
            elif start < len(self.audio_track):
                remaining = len(self.audio_track) - start
                audio[:remaining] = self.audio_track[start:]
            # Advance cursor (advance by samples_per_tick, not full chunk, for overlap)
            self._audio_cursor += samples_per_tick

        return self._current_frame, audio

    def release(self):
        if self.cap:
            self.cap.release()


def stream_from_video(video_path: str, loop: bool = True):
    """Stream frames from video file at 90 Hz."""
    source = VideoSource(video_path, loop=loop)
    frames_sent = 0

    try:
        while True:
            start = time.perf_counter()

            rgb, audio = source.next_frame()
            if rgb is None:
                print(f"Video ended after {frames_sent} SENS frames", file=sys.stderr)
                break

            write_frame(rgb, audio)
            frames_sent += 1

            # Progress every 10 seconds
            if frames_sent % (TARGET_FPS * 10) == 0:
                elapsed_min = frames_sent / TARGET_FPS / 60
                print(f"  Streamed {frames_sent} frames ({elapsed_min:.1f} min)", file=sys.stderr)

            # Maintain 90 Hz
            elapsed = time.perf_counter() - start
            sleep_time = max(0.0, FRAME_INTERVAL - elapsed)
            if sleep_time > 0:
                time.sleep(sleep_time)

    except (KeyboardInterrupt, BrokenPipeError):
        pass
    finally:
        source.release()
        print(f"Optic Nerve stopped. Total frames: {frames_sent}", file=sys.stderr)


def stream_from_desktop():
    """Stream frames from desktop capture at 90 Hz."""
    try:
        import mss
    except ImportError:
        print("ERROR: mss not installed. pip install mss", file=sys.stderr)
        sys.exit(1)

    print(f"Streaming desktop at {TARGET_FPS} Hz", file=sys.stderr)

    try:
        with mss.mss() as sct:
            monitor = sct.monitors[1]
            while True:
                start = time.perf_counter()

                screenshot = sct.grab(monitor)
                img = np.array(screenshot)
                img_rgb = cv2.cvtColor(img, cv2.COLOR_BGRA2RGB)
                img_resized = cv2.resize(
                    img_rgb, COGNITIVE_RESOLUTION, interpolation=cv2.INTER_LINEAR
                )

                audio = np.zeros(AUDIO_SAMPLES_PER_FRAME, dtype=np.float32)
                write_frame(img_resized, audio)

                elapsed = time.perf_counter() - start
                sleep_time = max(0.0, FRAME_INTERVAL - elapsed)
                if sleep_time > 0:
                    time.sleep(sleep_time)

    except (KeyboardInterrupt, BrokenPipeError):
        pass


def main():
    parser = argparse.ArgumentParser(
        description="Optic Nerve - Transducer for external video/desktop capture (90 Hz output)"
    )
    parser.add_argument(
        "--source",
        type=str,
        help="Video file path (mp4, avi, mkv, etc.)"
    )
    parser.add_argument(
        "--desktop",
        action="store_true",
        help="Capture desktop instead of video file"
    )
    parser.add_argument(
        "--no-loop",
        action="store_true",
        help="Don't loop the video (stop at end)"
    )

    args = parser.parse_args()

    if args.source:
        source_path = Path(args.source)
        if not source_path.exists():
            print(f"ERROR: Video file not found: {source_path}", file=sys.stderr)
            sys.exit(1)
        stream_from_video(str(source_path), loop=not args.no_loop)
    elif args.desktop:
        stream_from_desktop()
    else:
        print("ERROR: Specify --source <video_file> or --desktop", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
