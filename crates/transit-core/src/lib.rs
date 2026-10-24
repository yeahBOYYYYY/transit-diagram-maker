//! Core of Transit Diagram Maker: a grid based, topologically aware document
//! model for metro maps with undo/redo, `.transit.svg` files and SVG output.

pub mod assets;
pub mod editor;
pub mod error;
pub mod geom;
pub mod icons;
pub mod io;
pub mod model;
pub mod render;
pub mod tree;

pub use assets::{Assets, FontAsset, IconAsset};
pub use editor::{CreatedLine, CreatedStation, Editor, LineHit, StationOptions, TreeItem};
pub use error::{EditError, Result};
pub use geom::{Cell, Dir8, Rect, RouteMode, Vec2};
pub use model::*;
