vhrdcan
=======

FrameId
-------
Checked CAN Bus identifier, can be either StandardId or ExtendedId.
Constructors are const fn, so you can check that IDs are valid during compile time:
```rust
const MOTOR_DRIVE_ID: FrameId = FrameId::new_extended(0x1).unwrap();
const EMERGENCY_STOP_ID: FrameId = FrameId::new_standard(0x0).unwrap();
```

RawFrameRef
-----------
Contains FrameId + reference to a [u8] slice.
Can be used to create a frame without copying the data around.

RawFrame
--------
Contains FrameId + owned [u8; 8] array + length to determine an actual amounts of bytes used.

Frame
-----
Contains RawFrame + sequence number. Can be safely created only from `FramePool` to preserve message order and avoid various bugs such as priority inversion (CAN Bus driver should be carefully checked for priority inversion also, especially if it has several transmit buffers).

FramePool
---------
Allows creation of Frame's while preserving their creation time.

Traits
------
`Ord`, `Hash`, `hash32::Hash`, nice `Debug`, `Eq`, `PartialEq`, `Copy`, `Clone` is implemented.
`Serialize` and `Deserialize` is behing a `serialization` feature gate.