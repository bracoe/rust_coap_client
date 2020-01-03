#!/bin/bash

#Check if Storage file exists
FILE=./Storage
if [ -d "$FILE" ]; then
    echo "PASSED: $FILE exist!"
else 
    echo "FAILED: $FILE does not exist!!!"
    exit 1
fi

#Check if sensor1 exists
coap-client -m post coap://localhost:5683/sensor1

sleep 2

FILE=./Storage/sensor1
if [ -f "$FILE" ]; then
    echo "PASSED: $FILE exist"
else 
    echo "FAILED: $FILE does not exist and should!!!"
    exit 1
fi

#Check file is removed
coap-client -m delete coap://localhost:5683/sensor1

sleep 2

if [ -f "$FILE" ]; then
    echo "FAILED: $FILE should not exist !!!"
    exit 1
else 
    echo "PASSED: $FILE does not exist"
    
fi

#Check if sensor1 exists
coap-client -m post coap://localhost:5683/sensor1

sleep 2


if [ -f "$FILE" ]; then
    echo "PASSED: $FILE exist"
else 
    echo "FAILED: $FILE does not exist and should!!!"
    exit 1
fi

#Check if sensor1 exists
echo -n "mode=on" | coap-client -m put coap://localhost:5683/sensor1 -f-

sleep 2

if [ -s "$FILE" ]
then
    echo "PASSED: $FILE contains something!"
else
    echo "FAILED: $FILE does not contain anything!!!"
    exit 1
fi

# echo -n "mode=on" | coap-client -m put coap://localhost:5683/sensor1 -f-
