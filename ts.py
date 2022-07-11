import os
from pickle import FALSE, TRUE;


file_name = './user/src/bin/2.rs'
(pair_num_pos, len_pos) = (43, 46)
result_file_name = 'user_cor_x.txt'

def set_args(pair_num, len):
    lines = []
    with open(file_name, 'r+') as f:
        lines = f.readlines()
        lines[pair_num_pos - 1] = str(pair_num) + '\n'
        lines[len_pos - 1] = str(len) + '\n'
    
    with open(file_name, 'w+') as f:
        f.write(''.join(lines))
    return



X = [2, 50, 100, 200, 400, 600, 800, 1000, 
    1200, 1400, 1600, 1800, 2000, 2200, 2400, 
    2600, 2800, 3000, 3200, 3400, 3600, 3800, 4000]

Y = [200, 400, 600, 800, 1000, 1200, 1400, 
    1600, 1800, 2000, 2200, 2400, 2600, 2800, 
    3000, 3200, 3400, 3600, 3800, 4000]

text = ''

""" for x in X:
    for y in Y:
        if y >= x and y % x == 0:
            ok = FALSE
            while ok == FALSE:
                set_args(y/x, x)
                output = os.popen("python 1.py").read()

                output_lines = output.split("\n")
                for s in output_lines: 
                    line = s.split(' ')

                    if line[0] == '>>>':
                        print(str(x) + ' ' + str(y/x) + ' ' + line[1])
                        text = text + '\n# ' + str(x) + ' ' + str(y) + '\n' + line[1] + '\n'
                        ok = TRUE
                        break """

# 重复测试次数
REPEAT_NUM = 1

def calc_avg(arr):
    return sum(arr) / len(arr)

for x in X:
    for y in Y:
        if y >= x and y % x == 0:
            set_args(int(y/x), int(x))

            times = []
            while len(times) < REPEAT_NUM:
                ok = FALSE
                
                output = os.popen("python 1.py").read()

                output_lines = output.split("\n")
                for s in output_lines: 
                    line = s.split(' ')
                    if line[0] == '>>>':
                        print(str(x) + ' ' + str(y/x) + ' ' + line[1])
                        #text = text + '\n# ' + str(x) + ' ' + str(y) + '\n' + line[1] + '\n'
                        ok = TRUE
                        break
                if ok == TRUE:
                    times.append(int(line[1]))

            text = text + '\n# ' + str(x) + ' ' + str(y) + '\n' + str(calc_avg(times)) + '\n'
            

text = text + '\n\n\n\n\n'
with open(result_file_name, 'w+') as f:
    f.write(text)