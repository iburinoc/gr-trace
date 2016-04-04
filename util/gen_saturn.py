#!/usr/bin/python

import noiselib
import numpy as np
from PIL import Image

size = (1000, 915)

s_col = Image.open("../resources/saturnringcolor.jpg", 'r')
s_trans = Image.open("../resources/saturnringpattern.gif", 'r')

out_img = Image.new('RGBA', size, (0, 0, 0, 0))

for y in range(0, size[1]):
	o_col = s_col.getpixel((y, 32))
	o_trans = s_trans.getpixel((y, 32))
	col = (o_col[0], o_col[1], o_col[2], o_trans)
	for x in range(0, size[0]):
		out_img.putpixel((x, y), col)


out_img.save("out.png", "PNG")

