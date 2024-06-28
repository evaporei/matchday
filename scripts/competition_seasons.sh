#!/usr/bin/env bash

API_KEY=$1

curl --request GET \
     --url "https://api.sportradar.com/soccer/trial/v4/en/competitions/sr%3Acompetition%3A17/seasons.json?api_key=${API_KEY}" \
     --header 'accept: application/json'
