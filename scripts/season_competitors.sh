#!/usr/bin/env bash

API_KEY=$1

curl --request GET \
     --url "https://api.sportradar.com/soccer/trial/v4/en/seasons/sr%3Aseason%3A105353/competitors.json?api_key=${API_KEY}" \
     --header 'accept: application/json'
