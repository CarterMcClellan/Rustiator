game_id=$(curl -X POST -s localhost:8080/new_game -d '{"mode":"playerVsBot"}' -H "Content-Type: application/json" | jq -r .game_id)

board_state=$(curl -X POST -s localhost:8080/play/$game_id -d '{"move":"e2e4"}' -H "Content-Type: application/json" | jq -r .board_state)

open "http://www.ee.unb.ca/cgi-bin/tervo/fen.pl?select=$board_state"