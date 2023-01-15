# LVL: Layer Varied Layout, Format Version 0
This document is meant to describe the Layer Varied Layout format, and will be referred to as LVL furthermore. LVL is a uncompressed binary format, used to contain the main building blocks that a typical game scene will contain.

All files using the LVL format will have `LVL Format 0.` written at the start of the file as the header. This header's backing type is a UTF-8 String, and all references to a "String" type should be assumed UTF-8 by default.

___

## Define Global Room Properties
| Offset | Type   | Description            |
|--------|--------|------------------------|
| 0x0000 | UInt8  | Command Code, always 0 |
| 0x0001 | UInt64 | Room Width             |
| 0x0002 | UInt64 | Room Height            |
| 0x0003 | Int64  | X Offset*              |
| 0x0004 | Int64  | Y Offset*              |
___

## Set Layer Properties
| Offset | Type               | Description            |
|--------|--------------------|------------------------|
| 0x0000 | UInt8              | Command Code, always 1 |
| 0x0001 | UInt32             | Cell Width             |
| 0x0002 | UInt32             | Cell Height            |
| 0x0003 | Int64              | X Offset*              |
| 0x0004 | Int64              | Y Offset*              |
| 0x0005 | Layer Type (UInt8) | Layer Type             |
Layer Types:
0. Tile
1. Grid
2. Decal
3. Entity
4. Unknown

___

## Grid Cell
| Offset | Type  | Description            |
|--------|-------|------------------------|
| 0x0000 | UInt8 | Command Code, always 2 |
| 0x0001 | Int16 | X Position             |
| 0x0002 | Int16 | Y Position             |
| 0x0003 | Int8  |                        |
### Notes
The "Arbitrary Cell Type" here refers to the value that is usually defined in the application generating the original manifest. This arbitrary value is then interpreted by the application processing the binary LVL format, that then chooses what to do with each value.

—

## Create Entity
| Offset | Type            | Description            |
|--------|-----------------|------------------------|
| 0x0000 | UInt8           | Command Code, always 3 |
| 0x0001 | Int32           | Entity Ref             |
| 0x0002 | Int32           | X Position             |
| 0x0003 | Int32           | Y Position             |
| 0x0004 | UInt32          | Width                  |
| 0x0005 | UInt32          | Height                 |
| 0x0006 | Int16           | Rotation               |
| 0x0007 | Boolean (UInt8) | Flipped X Bit          |
| 0x0008 | Boolean (UInt8) | Flipped Y Bit          |
| 0x0009 | Int32           | X Origin*              |
| 0x0010 | Int32           | Y Origin*              |
—

## Add Tile
| Offset | Type   | Description            |
|--------|--------|------------------------|
| 0x0000 | UInt8  | Command Code, always 4 |
| 0x0001 | Int32  | Tileset Asset Id       |
| 0x0002 | UInt32 | X Position             |
| 0x0003 | UInt32 | Y Position             |
| 0x0004 | UInt16 | Tile X Position        |
| 0x0005 | UInt16 | Tile Y Position        |
| 0x0006 | UInt16 |                        |
---

`*`:  Unimplemented, reserved for future use. Pass in the actual value, or just leave it at a default value, but never omit it.