# Nursery Videos Directory

This directory contains educational video files for the Quaternity organism to learn from.

## Setup Instructions

1. **Download Educational Videos**
   - Place `.mp4` video files in this directory
   - Recommended content:
     - `lesson_01_phonics.mp4` - Sesame Street "Letter A" or similar phonics lessons
     - `calibration_nature.mp4` - Planet Earth or nature documentary clips
     - `calibration_chaos.mp4` - High-entropy content (e.g., Koyaanisqatsi clips)

2. **File Naming**
   - Use descriptive names: `lesson_01_phonics.mp4`, `nature_forest.mp4`, etc.
   - The organism will learn from whatever videos are present

3. **Video Requirements**
   - Format: `.mp4` (H.264 video, AAC audio recommended)
   - Resolution: Any (will be resized to 640x480 in broadcast window)
   - Duration: Any length

## Running the Broadcast

From the project root:

```bash
# Install Python dependencies
pip install -r scripts/nursery/requirements.txt

# Play all videos in this directory
python scripts/nursery/broadcast.py

# Play specific videos
python scripts/nursery/broadcast.py lesson_01_phonics.mp4 calibration_nature.mp4

# Loop a single video
python scripts/nursery/broadcast.py --loop lesson_01_phonics.mp4

# Loop all videos
python scripts/nursery/broadcast.py --loop-all
```

## How It Works

1. The Python script plays videos in a fixed window at position (100, 100) with size 640x480
2. The Rust organism's Retina captures the screen and detects high optical flow (motion)
3. The high-motion region (the TV window) is identified as the "Nursery Broadcast"
4. The Observer focuses attention on the TV center
5. The Memory system records correlations between visual patterns and audio sounds

## Notes

- The organism learns through **literal experience** - no synthetic data
- Videos should have clear visual-audio correlations (e.g., letter "A" + "Ah" sound)
- The organism may need multiple viewings to form strong correlations
- Keep the broadcast window visible and not covered by other windows

