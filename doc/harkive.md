# harkive

A command-line Haken EaganMatrix preset archiver.

Loads and saves presets from any device with Haken Audio's EaganMatrix engine.
Due to historic Windows limitations, it cannot be used while the Haken Editor is running.

## Usage

hem-archive is a non-interactive console program that must be run using a command prompt or batch script.

The command-line syntax is:

**harkive** \[**--device** *name*] *action* \[*path*]

Square brackets indicate an optional item. They are not used in an actual command line.
Exactly one *action* is required for each run of the program.

| Option/Action | Shorthand | Description |
| -- | :--: | -- |
| **&#x2011;&#x2011;device**&nbsp;*name*  | **-d** | Name of device to save/restore from. The device name can be a partial name as long as it is sufficiently unique. For example, `-d Mini` is often sufficient to find a ContinuuMini, even if other EaganMatrix devices are connected. If no device name is given, the first suitable device is used. |
| **&#x2011;&#x2011;input**    | **&#x2011;i** | Print list of connected MIDI devices. |
| **&#x2011;&#x2011;monitor**  | **&#x2011;m** | Log MIDI received from the selected device. |
| **&#x2011;&#x2011;clear**    | **&#x2011;c** | Clear all user presets from the device. |
| **&#x2011;&#x2011;print**    | **&#x2011;p** | Print list of user presets. |
| **&#x2011;&#x2011;edit**     | **&#x2011;e** | Save current editing slot. |
| **&#x2011;&#x2011;save**     | **&#x2011;s** | Save user presets from the device to *path*. |
| **&#x2011;&#x2011;load**     | **&#x2011;l** | Load user presets from *path* to the device. |
| **&#x2011;&#x2011;help**     | **&#x2011;h**, **&#x2011;?** | Help. The short forms print short help. |

Preset lists are similar to Haken Editor group lists.

*path* usage:

**--input**, **--monitor**, and **--clear** do not use *path*.

*path* is required to load or save. The folder of the path must exist on disk.

*path* can generally be either a file path or a folder. When no file name is
given, a default name is assumed or generated.

**--save**: When *path* is a file path, it is a preset list and the preset .mid
files go to the same folder. The default file name is `UserPresets.txt`.

**--edit**: If *path* ends with *name*`.mid`, the editing slot midi data is written
to that filename. Otherwise, *path* is a folder. If the slot is unnamed or
"Empty", a unique filename is generated in the format "`anon-`*NNNN*`.mid`" using
a hash of the preset midi data.

**--load**: When *path* is a file name, it is either preset list (.txt), or a
preset midi data (.mid) file. For a .mid file, the preset is loaded into slot
zero, the editing slot. For a preset list file, the preset .mid files are
expected in the same folder. The preset numbers are interpreted as absolute
preset slot numbers from 1 to 128. When *path* is a folder, if the folder
contains a `UserPresets.txt` file, that list file is used. Otherwise all preset
.mid files in the folder are loaded in alphabetical order.

## Listing format

hem-archive uses a preset listing format compatible with the Haken Editor group file format.
Any leading space on a line is ignored.

```text
14,"Mojo of FDN.mid"
 13,"Marlin Perkins 1.mid"
 12,"Ishango Bone.mid"
 11,"Bowed Mood.mid"
 10,"Living Pad.mid"
 9,"Shimmer.mid"
 8,"Cumulus.mid"
 7,"Choir.mid"
 6,"Bass Monster.mid"
 5,"Snap Bass.mid"
 4,"Jaymar Toy Piano.mid"
 3,"Woodwind.mid"
 2,"Tin Whistle.mid"
 1,"Vln Vla Cel Bass 2.mid"
```

## Reference: Groups

| Group # | Preset Range |
| :--: | -- |
| 1 | 1-16 |
| 2 | 17-32 |
| 3 | 33-48 |
| 4 | 48-64 |
| 5 | 65-80 |
| 6 | 81-96 |
| 7 | 97-112 |
| 8 | 113-128 |
