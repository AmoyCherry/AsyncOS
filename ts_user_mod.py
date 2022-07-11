import os
from pickle import FALSE, TRUE;


file_name = './user/src/bin/5.rs'
len_pos = 32
result_file_name = 'user_mod.txt'

def set_args(len):
    lines = []
    with open(file_name, 'r+') as f:
        lines = f.readlines()
        lines[len_pos - 1] = str(len) + '\n'
    
    with open(file_name, 'w+') as f:
        f.write(''.join(lines))
    return



Y = [200, 400, 600, 800, 1000, 1200, 1400, 
    1600, 1800, 2000, 2200, 2400, 2600, 2800, 
    3000, 3200, 3400, 3600, 3800, 4000]

text = ''

for y in Y:
    ok = FALSE
    while ok == FALSE:
        set_args(y)
        output = os.popen("python 1.py").read()

        output_lines = output.split("\n")
        for s in output_lines: 
            line = s.split(' ')

            if line[0] == '>>>':
                print(str(y) + ' ' + line[1])
                text = text + '\n# ' + ' ' + str(y) + '\n' + line[1] + '\n'
                ok = TRUE
                break

text = text + '\n\n\n\n\n'
with open(result_file_name, 'w+') as f:
    f.write(text)

