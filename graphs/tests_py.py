import re

line = 'CHN(0x564d74bb5008)" [label="CHN" color="black" comment="[108,592,0,108,0,108,108,176,108,0,40,0,108,0,33,0,65,138,202,0,0,108,0,1,84,0,337,0,0,0,1,0,0,249,0,0,0,94890670903304,2.355388542207534]"]'
result = re.sub(r' comment=".*?"', '', line)

print(result)