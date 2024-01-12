---Create your bot as a table
MyRandomBot = {}

---Your bot must have a chooseMove method
---Note that the `:` notation provides self as an implicit argument
---@param chess_game any
---@param legal_moves string[] list of legal moves in uci notation
---@return string move to make in uci notation
function MyRandomBot:chooseMove(chess_game, legal_moves)
    return legal_moves[ math.random( #legal_moves ) ]
end

---Must return your bot from the script
return MyRandomBot