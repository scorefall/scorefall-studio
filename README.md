# ScoreFall™ Studio
A Digital Audio Workstation (DAW): Record and mix tracks into a final recording.

## TUI
Screenshots: TODO

## GUI
TODO: Doesn't exist yet.

# ScoreFall Multitrack Audio Format (*.sma)
Source audio is broken up into 30 second chunks (compressed separately).

 - Version: 0u16
 - Metadata: List (one of:)
   - Title 0: Text
   - Artist 1: Text
   - Album 2: Text
   - Track 3: u8
   - Release 4: DateSubset
   - Genre 5: Text
   - Comments 6: Text
   - CoverArt 7: Raster
   - VectorArt 8: VectorGraphics
   - Unknown 9-254
   - Custom 255: \[(Text, Text)\]
 - Source Audio Tracks: List (upto 65,535)
   - Freq: u32
   - Chan: u3 (offset -1)
   - Used: bool
   - Part: u12 (which 30 seconds)
   - Name: Text
   - Data: \[f32\]
 - Destination Audio Tracks: List (upto 65,535)
   - Name: Text
   - Group: u16
   - Source: u16
   - When: u32 (sample index in 48K audio)
