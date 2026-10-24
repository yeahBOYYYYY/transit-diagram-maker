# Bugs & Teaks

## 24/10/2026 - Commit db38ed1

1. All line segments of the same line must share a style, this is weird since there are lines in which the style will change along the line, in those cases we want to seperate the line into segments with different styles.
2. When exporting to SVG it should show settings about the export and not all the project settings. Check what settings are actually relevant!
3. Usage of shortening of words in any title is a bad habit! It makes the reader must have context, hence all strings must not contain shortcuts that aren't part of the English language (like "isn't").
4. It might be better if the GitHub logo should be in the bottom bar on the right. And the save icon on the top bar on the left, right to the right of the "Td" logo.
5. The logo of the page in the browser and the logo inside in the menu bar should be the same, I like the favicon more.
6. Selecting icons is done quite bad, instead of a big dropdown menu, you can transform the entire tool into a separate panel tab next to "Properties", this panel will show all available icons in an ordered grid fashion and the same option to upload a new SVG like now. Then it will not feel cramped in the tiny layout of the app. Also it will be easier to also be able to drag and drop an icon from that panel directly (while also keeping the option to select it via clicking).
7. I'm not sure `.transit` should be a JSON, won't it be better to be a flavour of XML (so that all the common keys would just be a part of a tag).

## 24/10/2026 - Commit 2c92670

Those are the bugs needed to be fixed,

1. Everything including text should snap to 45-degree angle multiples when rotating.
2. Free move mode for stations doesn't work.
3. Tick style stations are snapped to the center of a cell instead of being snapped to an inner intersection of two axis lines (meaning it should have exactly two sides intersecting with the grid lines).
4. The "Adjustments" panel doesn't do anything and doesn't work. Remove if not needed or give it a purpose.
5. The "Paths" panel doesn't do anything and doesn't work. Remove if not needed or give it a purpose.
6. Allow the user to upload his own text fonts.
7. Allow the user to upload his own SVG icons.
8. Change the icon for "Icon Tool" to actually represent it's meaning.
9. When long pressing on the Pen Tool it should show the hidden tools inside it, likewise when double clicking, not only when right clicking.
10. There are tabs in the main window of the web app for multiple possible `.transit` files, but you can't actually have more then one open concurrently (need to decide if want concurrent use or remove tabs, in my opinion we should stick to one file at a time).
11. The dashed line style doesn't work well, it has weird artifacts near the end of the line, the text in the properties panel is confusing and the input boxes for all the different parameters don't work as expected. (Need major reprogramming)
12. Unable to get two disconnected line segments to be in the same line layer group, not by dragging or by creating while focusing on the other's layer.
13. Station pill style doesn't strech out to any direction. It should be completely "modular", meaning that visually it's a group of blobs with merged inner filling and one merged border around them. In terms of actual implementation you can do that instead of having four corners that you drag to scale it, it will have a small arrow pointing to any adjacent cell (no matter the form of the blob) and when the user picks an arrow it grows into it (picks meaning double clicks for example).
14. When exporting to SVG it should show the same settings as in the default settings menu for the user to choose (maybe they want one time custom settings).
15. The panel for properties should not grow with the amount of options present, it should stay always in place unless the user drags the barrier between the panels.
16. All properties shown need to have help information (when hovering on them or using an information icon beside, it should be consistent).
17. Make the grid precision be only the first digit after the point (meaning that objects will not be allowed to have precision of 0.01 but rather 0.1).
18. All objects on the canvas will have in their properties an X and Y coordinates (to allow for easier snapping to other objects manually).
19. Remove the default setting for "Cell size". The cell size is not relevant to any object since everything is measured relative to it. Instead when exporting to SVG allow the user to pick the "cell length" to "pixel amount" conversion rate, with default being 24 as it is now!
20. The about section should contain data about the program itself in term of general information about the author and github repo and such, not the underlying frameworks used!
21. Add an icon for GitHub in the right side of the menu bar with URL to the github repo, also add a Save icon on the same side for fast `.transit` export (maybe we can cache where was it saved after first save then just save it again after to the same file).
