#!/bin/bash

coap-client -t 00 -m put coap://localhost:5683/sensor1

echo -n "mode=on" | coap-client -m put coap://localhost:5683/sensor1 -f-