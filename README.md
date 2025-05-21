# mucky-physics
An extremely early demo of mass aggregate physics with Verlet integration, using Zig and Raylib.
I have a lot of fun projects and this one is probably close to the bottom of my list.
I'm fairly new to Zig, and just wanted to make a quick project to test things out.
This isn't going to be a physics engine or anything, but I do want to expand it's functionality to being a fully-fledged physics playground.
That's a fair distance into the future, though.

## Future plans:
- Allow particles and restraints to collide with each other. This is a must-have,
but I'm still traumatized from my past attempts at implementing this...
- Letting the user grab onto whatever particle or constraint they want.
- An immediate mode GUI, probably RayGui, because there's no way I'm making my own UI.
It'll actually be somewhat outside my comfort zone, because I'm rather used to DearImGUI instead.
This actually includes the addition of a lot of tweakable settings, but I'm not going to write those out.
- This will include being able to generate new particles and constraints.
- This is somewhat out there, but being able to create different levels,
where the geometry is made up of a series of line segments,
and being able to dynamically save, load, and create these levels.

Having these things, except for maybe that last one, would put us in a really good place!
