# Agnes's projected shadow

Each animated pose owns nine shadow sprite children. They reuse that pose's
texture, alpha silhouette, clips, timing and playback speed. Disabling the pose
also disables its shadows through the hierarchy. Player.show starts and restarts
the pose and its nine shadows together, including both directions of the turn.

The sheet's bottom anchor is the foot of both the character and the projection.
A local Y scale of -0.34 flips the silhouette toward the foreground and compresses
it onto the floor. It inherits Agnes's movement and scale. Z remains above the
background, and layer 5 draws below the character's layer 10.

Nine faint copies in a 3x3 arrangement soften the silhouette's edge without new
textures or an engine change. This is an approximation of blur using alpha
compositing, not a shadow map or a GPU blur filter. Only the active pose's nine
shadow sprites draw. The central opacity is 0.09, axial copies 0.045, and diagonal
copies 0.0225; each sample is spaced 0.01 local units apart.

All shadows must retain identical projection transforms except for those sample
offsets. Their names follow "Agnes Shadow <clip> <sample>", with "turn" in place of
the clip name for the shared turn entity. Changing a clip's timing or frames
requires the same change in its shadow children.
