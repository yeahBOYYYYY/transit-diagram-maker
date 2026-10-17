A specialized, high-performance vector editor designed to construct complex, topologically sound metro and transit maps. The application is architected around a procedural node-graph paradigm, powered by a **Rust** backend compiled to **WebAssembly (WASM)** with a **Vue 3 / TypeScript** dark-mode web presentation layer.

I want to build a website that is a "Transit Diagram Maker". It will be built using React, TailwindCSS and Rust WASM.

## Appearance

The website should look like a product from "Adobe suite" (more specifically with astetics similar to Photoshop).

- It will feature on the left size a "Tools panel" which will hold a couple of tools (explained in the next chapter).
- It will feature a right panel "Properties panel" and "Layers panel".
- Menu bar with options to change pretty much anything in regards to the "constants" of the application (any thing that is arbitrary should be able to change here).

## Functionality

### Background

The "canvas" on which the diagrams are to be drawn is a square 2d grid.

### Tools

Each tool has it's on shortcut that can be changed in the settings, and is mostly a single letter to make the work as productive as possible.

#### Selection tool

The "selection tool" can be used to select on the canvas an object (a layer) on the canvas, or multiple objects at once.

Using the tool we can move those object after selecting them, or by clicking on them be able to rotate them all simultaneously.

#### Line pen tool

The "line pen tool" is used to draw a new line. A line is made up of two possible orientations,

- 1x1 grid cells (meaning the line itself is the grid cells and not on their intersection) in a single direction.
- sloped grid cells, a line with a slope that is a multiple of 45 degrees, that starts and ends is a 1x1 grid cell.

when a line transitions between the two orientations it will do so with a nice curvy transition (the default can be changed in the settings as do all other arbitrary constants).

Each continuous portion of a line is considered to be a different layer, and all the different discontinuous segments of the line are gathered in a "layer group".

A line may end (and start) using two possible caps,

1. A "butt cap", which is a flat non existing ending, meaning that the line just ends once on the last grid cell.
2. A "round cap", the last grid cell that the line occupies will not be full and will feature a semi-circle to close of the line. (the roundness is again something that can be changed)

A line has a few different styles,

1. The "plain style", a solid color line.
2. The "hollow style", a solid color line with an inner solid color. The width of the inner color can be changed using percentages.
3. The "dashed style", a solid color line with an inner dash of the same color. The spacing between dashes can be changed, the frequency of the dashes can be changed, the width of the dashes can be changed, and the offset from the start of the line can be changed.

#### Erase tool

The erase tool allows to erase portions of a line to make it a discontinuous line, meaning that it will also split it into different layers under the same "layer group".

This tool can be combined with the "Line pen tool" to be in the same panel slot and be shown only on long presses on the pen tool or via it's keyboard shortcut.

#### Station tool

Once a line is created, it can be populated with stations.

A station has two possible styles,

1. A "dot station" is made up of two circles (one inner and one outer) that each has a different color. Defaultly the outer one is black and the inner is white, and their total size is a bit more than a 1x1 grid cell (to make it stand out against the line).
2. A "tick station" is a thin rectangle poking out of the line in the specified direction. As it's not a square it should be oriented the same way the line is, such that the base of the tick station is a part of the line's border. The width, height and color of it may be changed.

A station must be coupled to a line, and when a station is created it will be added to the line's "layer group" automatically (the group will contain the line itself and it's stations).

#### Text tool

Allows the user to plant text boxes anywhere on the canvas.

When planting a new station on the canvas it will also make a new text box along side it, which will be coupled to the station as it's the station's name. This textbox can be removed as it's a different layer then the station itself but they are in a "layer group" when spawning.

### Compatibility

The website will provide two possible ways to save work, it can be either exported to SVG, or it can be exported into `.transit` which is a file that can be imported back into the website (lossless serialization format, JSON or XML or something like that)
