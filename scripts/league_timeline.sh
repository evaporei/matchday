#!/usr/bin/env bash

API_KEY=$1

curl --request GET \
     --url "https://api.sportradar.com/soccer/trial/v4/en/sport_events/sr%3Asport_event%3A41762859/league_timeline.json?api_key=${API_KEY}" \
     --header 'accept: application/json'
