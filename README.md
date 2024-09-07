# Eileda

SDL2-based presentation software.

## What?

This is an experimental slideshow program built for personal use, inspired by
100r's adelie.

## Why?

<img align="right" width="40%" style="float: left; padding-left: 20px" src="https://0x0.st/XwQE.gif" />

I created this mostly because I'm attracted to the idea of extremely simple,
barebones presentation software, such as suckless' sent or adelie, but would
still like some highly advanced/exotic typesetting capabilities (like bold
text).

Also, because Biden has mandated that all software engineers must either produce
a static site generator, or a presentation app, at some point in their life.

## How?

Eileda uses a frankenstein system of Markdown with TROFF-style directives
sprinkled in between.

See `text.eimd` for example.

Configuration directives (must appear before content):

- `.PAD`: Slide padding, in pixels.
- `.MAR <middle> <image_path>`: Creates a box of size `<middle>` (pixels),
  centers it on the screen, and displays `<image_path>` in the margins, if any.

Content directives:

- `.SLD`: Begin a new slide. At least one of these must exist.
- `.GRD <ratio>`: Begin a two-column grid. Cannot be nested. `<ratio>` is a
  number in between 0 and 100. Example: `.GRD 40` creates a grid where the first
  column is 40% of the width, and the second is 60%.
- `.COL`: Begin next column in grid.
- `.IMG <path>`: Embed an image. Note that images are "greedy" and take up
  available space. Known issues: content underneath an image doesn't appear.

## License

Eileda bundles a number of font files in `assets/`. I did not create those and I
take no credit for them. However, I did make minor modifications to a few `ufx`
fonts, such as changing the character alignments.

- [Ufx Font License (MIT)](https://git.sr.ht/~rabbits/turye/tree/main/item/LICENSE)
  -- © Hundred Rabbits
- [Inter Font License (SIL)](https://github.com/rsms/inter/blob/master/LICENSE.txt)

Eileda itself is licensed under the MIT license. Feel free to fork and modify as
you please.
