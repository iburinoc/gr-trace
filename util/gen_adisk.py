#!/usr/bin/python

import noiselib
import numpy as np
from PIL import Image

size = (2000, 400)
nwidth = 5
min_col_ind = 175
max_col_ind = 375

noiselib.init(256)
xs = np.arange(0, nwidth, float(nwidth) / size[1])
ys = np.array(map(lambda z: noiselib.simplex_noise2((z, 0)), xs))
ys = (nwidth - xs) * ys
print ys
ys = ys / np.amax(ys)

bb_col = Image.open("../resources/bb-scale.jpg", 'r')

out_img = Image.new('RGBA', size, (0, 0, 0, 0))

for y in range(0, size[1]):
	val = ys[y]
	print val
	if val < 0:
		col = (0, 0, 0, 0)
	else:
		xind = val * (max_col_ind - min_col_ind) + min_col_ind
		ocol = bb_col.getpixel((xind, 25))
		col = (ocol[0], ocol[1], ocol[2], 255)
	for x in range(0, size[0]):
		out_img.putpixel((x, y), col)


out_img.save("out.png", "PNG")

