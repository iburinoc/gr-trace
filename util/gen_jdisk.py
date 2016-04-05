#!/usr/bin/python

import noise
import numpy as np
from PIL import Image

size = (2000, 2000)
ratio = 16

bb_col = Image.open("../resources/bb-scale.jpg", 'r')

out_img = Image.new('RGBA', size, (0, 0, 0, 0))

for y in range(0, size[1]):
	if y % 100 == 0:
		print y
	for x in range(0, size[0]):
		val = noise.snoise2(float(x)/size[0] * ratio,
			float(y)/size[1] * ratio,
			repeatx=ratio,
			repeaty=ratio)
		val = (val + 1) / 2.0
		val = val * 255
		val = int(val)
		col = (val, val, val, 255)
		out_img.putpixel((x, y), col)

out_img.save("out.png", "PNG")

