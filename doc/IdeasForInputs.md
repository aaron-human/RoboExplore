# How to do Inputs

I've hit a bit of a cross-roads: I'm not 100% sure how to handle jumping off of the "rails" in the game.

On one hand, want to make it similar to the existing jumping mechanics for jumping off of platforms (to make it intuitive): the control stick decides horizontal movement, while the length of time that the jump button is held determines the height.

But on the other hand, that would mean users could only jump upward, rather than being able to launch themselves in an arbitrary direction.  The current control scheme does allow "jumping" in an arbitrary direction off of the rails by having the jump button just start the jump, while the control stick decides the direction (and the "power" of the jump is fixed).

The inconsistency seems like a bad idea, but, at the same time, I don't see a sane was around it. Jumping off a platform implicitly always knows the direction the jump button is trying to send the player. Jumping off rails give the player far more directions to choose from.

TODO!