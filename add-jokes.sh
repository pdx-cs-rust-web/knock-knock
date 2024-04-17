#!/bin/sh
for F in assets/jokes/*/jokes/*.json
do
    curl -d "@$F" -H "Content-Type: application/json" -X POST http://localhost:3000/api/v1/joke/add
done
