#!/usr/bin/env python3
"""
Signal Generator - Environmental source for the Entity.

Generates raw binary sensory data (video/audio) mathematically and streams
to stdout at 90 Hz. Provides external field conditions for testing Reflection
and Regeneration axioms.

Output format: SENS protocol (type 1=Video, type 2=Audio).
"""

import sys
import struct
import time
import argparse
import math
from typing import Tuple

import numpy as np

# SENS Protocol constants
SENS_HEADER = b"SENS"
TYPE_VIDEO = 1
TYPE_AUDIO = 2
WIDTH, HEIGHT = 64, 64
AUDIO_SAMPLES = 2048
FPS = 90
INTERVAL = 1.0 / FPS

# Video payload size: 64*64*3 = 12288 bytes
VIDEO_PAYLOAD_SIZE = WIDTH * HEIGHT * 3
# Audio payload size: 2048*4 = 8192 bytes
AUDIO_PAYLOAD_SIZE = AUDIO_SAMPLES * 4

# Coherence mode
SINE_HZ = 432.0
CIRCLE_RADIUS = 8


def write_sens_chunk(chunk_type: int, payload: bytes):
    """Write a single SENS chunk: header + type + size + payload."""
    sys.stdout.buffer.write(SENS_HEADER)
    sys.stdout.buffer.write(bytes([chunk_type]))
    size = len(payload)
    sys.stdout.buffer.write(struct.pack("<I", size))  # u32 LE
    sys.stdout.buffer.write(payload)
    sys.stdout.buffer.flush()


def void_frame() -> Tuple[np.ndarray, np.ndarray]:
    """Null field: black frame and digital silence."""
    rgb = np.zeros((HEIGHT, WIDTH, 3), dtype=np.uint8)
    audio = np.zeros(AUDIO_SAMPLES, dtype=np.float32)
    return rgb, audio


def coherence_frame(t: float) -> Tuple[np.ndarray, np.ndarray]:
    """Calibration: moving geometric primitive (white circle) + 432 Hz sine."""
    # Figure-8 (Lissajous): center moves in 8 shape
    cx = 32 + 24 * math.sin(t)
    cy = 32 + 24 * math.sin(2.0 * t) * 0.5
    # Normalized position for audio modulation [0, 1]
    norm = (math.sin(2.0 * t) + 1.0) * 0.5

    rgb = np.zeros((HEIGHT, WIDTH, 3), dtype=np.uint8)
    for y in range(HEIGHT):
        for x in range(WIDTH):
            dx, dy = x - cx, y - cy
            if dx * dx + dy * dy <= CIRCLE_RADIUS * CIRCLE_RADIUS:
                rgb[y, x] = [255, 255, 255]

    # 432 Hz sine, amplitude modulated by circle position
    sample_phase = 2.0 * math.pi * SINE_HZ * np.arange(AUDIO_SAMPLES, dtype=np.float32) / 44100.0
    amplitude = 0.4 * (0.5 + 0.5 * norm)
    audio = (amplitude * np.sin(sample_phase)).astype(np.float32)
    return rgb, audio


def run_void():
    """Stream void frames at 90 Hz (SENS protocol)."""
    while True:
        start = time.perf_counter()
        rgb, audio = void_frame()
        
        # Write Video chunk (type 1)
        rgb_bytes = rgb.tobytes()
        write_sens_chunk(TYPE_VIDEO, rgb_bytes)
        
        # Write Audio chunk (type 2)
        audio_bytes = audio.tobytes()
        write_sens_chunk(TYPE_AUDIO, audio_bytes)
        
        elapsed = time.perf_counter() - start
        sleep_time = max(0.0, INTERVAL - elapsed)
        if sleep_time > 0:
            time.sleep(sleep_time)


def run_coherence():
    """Stream coherence frames at 90 Hz (SENS protocol)."""
    t = 0.0
    dt = 2.0 * math.pi * 0.5 / FPS  # slow figure-8 cycle
    while True:
        start = time.perf_counter()
        rgb, audio = coherence_frame(t)
        t += dt
        
        # Write Video chunk (type 1)
        rgb_bytes = rgb.tobytes()
        write_sens_chunk(TYPE_VIDEO, rgb_bytes)
        
        # Write Audio chunk (type 2)
        audio_bytes = audio.tobytes()
        write_sens_chunk(TYPE_AUDIO, audio_bytes)
        
        elapsed = time.perf_counter() - start
        sleep_time = max(0.0, INTERVAL - elapsed)
        if sleep_time > 0:
            time.sleep(sleep_time)


def main():
    parser = argparse.ArgumentParser(
        description="Signal Generator - Environmental source at 90 Hz"
    )
    parser.add_argument(
        "--mode",
        choices=["void", "coherence"],
        default="void",
        help="void: null field (zero energy). coherence: calibration (moving primitive + 432 Hz)",
    )
    args = parser.parse_args()

    if args.mode == "void":
        run_void()
    else:
        run_coherence()


if __name__ == "__main__":
    main()
