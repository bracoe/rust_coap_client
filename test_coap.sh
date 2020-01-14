#!/bin/bash

#Check if Storage file exists
FILE=./Storage
if [ -d "$FILE" ]; then
    echo "PASSED: $FILE exist!"
else 
    echo "FAILED: $FILE does not exist!!!"
    exit 1
fi

FILE=./Storage/sensor1
#Check non existing file cannot be deleted
if coap-client -m delete coap://localhost:5683/sensor1 |& grep -q "4.04"
then
    echo "PASSED: $FILE cannot be deleted as it does not exist and got error code 'Not Found'!"
else
    echo "FAILED: $FILE could be deleted even if it did not exist!!!"
    exit 1
fi

sleep 1

if [ -f "$FILE" ]; then
    echo "FAILED: $FILE should not exist!!!"
    exit 1
else 
    echo "PASSED: $FILE does not exist. "
    
fi

#Check if sensor1 exists
coap-client -m post coap://localhost:5683/sensor1 >> /dev/null

sleep 1

if [ -f "$FILE" ]; then
    echo "PASSED: $FILE exist"
else 
    echo "FAILED: $FILE does not exist and should!!!"
    exit 1
fi

#Check non existing file cannot be deleted
if coap-client -m post coap://localhost:5683/sensor1 |& grep -q "4.09"
then
    echo "PASSED: $FILE cannot be created again and got error code 'Conflict'!"
else
    echo "FAILED: $FILE could be created even if it existed already!!!"
    exit 1
fi

#Check file is removed
coap-client -m delete coap://localhost:5683/sensor1 >> /dev/null

sleep 1

if [ -f "$FILE" ]; then
    echo "FAILED: $FILE should not exist !!!"
    exit 1
else 
    echo "PASSED: $FILE does not exist"
    
fi

#Check if sensor1 exists
coap-client -m post coap://localhost:5683/sensor1 >> /dev/null

sleep 1


if [ -f "$FILE" ]; then
    echo "PASSED: $FILE exist"
else 
    echo "FAILED: $FILE does not exist and should!!!"
    exit 1
fi

#Check if sensor1 can receive value
STRING=hello
echo -n "${STRING}" | coap-client -m put coap://localhost:5683/sensor1 -f- >> /dev/null

sleep 1

if grep -q ${STRING} ${FILE};
then
    echo "PASSED: $FILE contains ${STRING} !"
else
    echo "FAILED: $FILE does not contain ${STRING}!!!"
    exit 1
fi

#Check if sensor1 can receive value

if coap-client -m get coap://localhost:5683/sensor1 | grep -q ${STRING}
then
    echo "PASSED: $FILE contains ${STRING}!"
else
    echo "FAILED: $FILE does not contain anything!!!"
    exit 1
fi


#Check if sensor2 cannot receive value
STRING=hello
echo -n "${STRING}" | coap-client -m put coap://localhost:5683/sensor2 -f-

sleep 1

if grep -q ${STRING} ${FILE};
then
    echo "PASSED: $FILE contains ${STRING} !"
else
    echo "FAILED: $FILE does not contain ${STRING}!!!"
    exit 1
fi

#Check file is removed
echo "Cleaning up!"
coap-client -m delete coap://localhost:5683/sensor1 >> /dev/null

sleep 1

if [ -f "$FILE" ]; then
    echo "FAILED: $FILE should not exist !!!"
    exit 1
else 
    echo "PASSED: $FILE does not exist"
    
fi
