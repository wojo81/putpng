# putpng

## Purpose
putpng is a sort of replacement for the Doom modding tools setpng and grabpng: https://zdoom.org/wiki/SetPNG

This program is only meant for offsetting with 'grAb' chunks within 'png' files, so it does not contain equivalents to setpng's 'alph', 'noalph', and 'z' options. However, it does contain a crop command which is present in other Doom modding tools.

## Usage

**grab** command: applies an offset to specified images by inserting or modifying a 'grAb' chunk.

    putpng grab <x> <y> <file_path(s)>
\
**crop** command: crop the empty edges out of the specified images and change the offsets of the images to match the relative positions of the original images (Note: the crop is destructive, so any chunks that are considered unnecessary will be removed)

    putpng crop <file_path(s)>
\
**show** command: show the offsets of the specified images

    putpng show <file_path(s)>
\
**ignore** option: optional argument that ignores any of the paths that contain any of the specified strings

    putpng <command> <file_path(s)> [-i | --ignore] <string(s)>
\
**help**: provides help

    putpng [-h | --help | help]

**version**: provides version

    putpng [-V | --version]

## Example

Let's say that a user wants to apply an offset of '(16, 32)' and crop a batch of sprites under various folders under the parent folder 'generic_weapon'. However, the folders named 'pickup' and 'projectile' need all of their underlying sprites to be centered. In this case they could open a terminal within the parent folder of 'generic_weapon' and write:

    putpng grab 16 32 generic_weapon\*\* -i pickup\ projectile\
    putpng grab w/2 'h / 2' generic_weapon\pickup\* generic_weapon\projectile\*
    putpng crop generic_weapon\*\*