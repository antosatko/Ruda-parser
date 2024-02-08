# Generates a workload for the string matching problem
#
# The data is generated in the following format:
#   - 10,000,000 lines
#   - Each line is 100 characters long
#   - This is a total of 1,000,000,000 characters or 1GB of data
#   - Each character is a random letter (a-z)
#   - Each line is terminated with a newline character
#   - Each line starts and ends with a double quote character (")
#
# The data is written to a file called workload.txt
# This file will be acompanied by a file called workload.meta
# The meta file contains the following information on each line of the file:
#   - Number of lines in the file
#   - Number of characters in each line
#


import random
import string
import time

lines = 10000000
lineLength = 100
fileSize = lines * lineLength


# Generate a random string of length n
def randomString(n):
    return ''.join(random.choice(string.ascii_lowercase) for i in range(n))

# Generate a random line of length n with 2 double quotes
def randomLine(n):
    return '"' + randomString(n - 2) + '"'

# Generate 1,000 lines of random data
def generateData():
    with open('workload.txt', 'w') as f:
        for i in range(lines):
            f.write(randomLine(lineLength) + '\n')

def generateMeta():
    with open('workload.meta', 'w') as f:
        f.write(str(lines) + '\n')
        f.write(str(lineLength) + '\n')

if __name__ == '__main__':
    print("Generating workload.txt")
    tm = time.time()
    generateData()
    print("Done in", time.time() - tm, "seconds")
    generateMeta()
    print("Done generating meta file")