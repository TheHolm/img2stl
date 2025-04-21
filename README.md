# img2stl

Yet another image to STL mesh converter, focused on creating STLs for CNC engraving.     

Creates an STL based on a two-colour engraving-style image: black for the image and white for the background.
If a colour image is provided, it will be converted to black and white, treating anything darker than 50% as black and the rest as white.
Images with an alpha channel are treated as if backed by 100% white.
Pixels outside the picture border are considered white.

## Usage

```
Usage: img2stl <input_file> [-o <output-file>] [-r <capture-radius>] [-p]

Positional Arguments:
  input_file        input file name

Options:
  -o, --output-file output file name. Default is "output.stl"
  -r, --capture-radius
                    look around radius. Use smaller values for sharper edges and
                    bigger for smoother. Default is 8 pixels. Doubling radius
                    quadripls compute time.
  -p, --generate-plane
                    if specified code will generate 0.001 unit thick base
                    underlying all other features. Default: Do Not Generate

```

## Releases

* deb is targeting Debian latst.
* standalone Linux binary should be run on any modern Linuxes. Dependencies are:
  ```
  linux-vdso.so.1
  libgcc_s.so.1
  libm.so.6
  libc.so.6
  ld-linux-x86-64.so.2
  ```
* Windows executable provided as best effort, have no means to test it.   
