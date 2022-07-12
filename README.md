# Proc Flow

Procedural Node Graph Content Creation.

The beginnings of something heinous.

## Attention

This project does not even have much code behind it yet. I am building out the spec so that I have a defined initial plan. Planning is necessary.

Right now the project can load libraries from the internal libraries in the root workspace as well as any libraries in your Documents folder.

Edit: I am now focusing development on the Arcana VR editor project. These two projects will be developed side by side, with ProcFlow as its core. Note that I still intend on building out editors for normal desktop mode operation. You should be able to double click and open a graph in Windows (or Linux when I get there in time) to edit that node graph. Node positions will still need to be in 3d, so I may have to extend it out so that 2d coord and 3d coords can both be stored.

## Build Notes

This project uses the shaderc crate, and because of such requires Python, CMake, and and C++ Compiler to be run by CMake. Also, it requires Ninja build on Windows. I don't like that anymore than you do, but unless someone writes a parser, AST generator, and SPIRV bytecode generator, and then puts them together, it will remain here until I am finished prototyping shaders. By all means I can just run shaderc on the GLSL shaders and actually just use bytecode in the programming, negating the need for the shaderc crate.

Please check the shaderc crate for more information.

# NOTE
I am actively going to rewrite and figure out some more pieces to this. I am going to start planning out some use cases to get this thing rolling.

THE FOCAL POINT OF PROJECT WILL INCLUDE THE ABILITY TO CONSTRUCT RUST PROGRAMS. I want this thing to self host itself, and allow you to directly create Rust nodal functionality on the fly. I have no idea what that will look like yet, but most likely, adhoc or library created nodes are going to build themselves as Rust projects as necessary unless I can find a way to directly script Rust in a performant way.
