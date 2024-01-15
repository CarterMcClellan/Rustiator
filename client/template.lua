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


-- Example 2. Down Moving bot
-- This always makes the move that will move a piece the farthest down
-- Look ahead = 1
MyDownBot = {}

function MyDownBot:chooseMove(chess_game, legal_moves)
    table.sort(legal_moves, MyDownBot.mostDown)
    return legal_moves[1]
end

function MyDownBot.mostDown(moveOne, moveTwo)
    return MyDownBot.amountDown(moveOne) > MyDownBot.amountDown(moveTwo)
end

---@param uci_move string
---@return number
function MyDownBot.amountDown(uci_move)
    local startingRow = uci_move:sub(2,2)
    local endingRow = uci_move:sub(4,4)
    return - (tonumber(endingRow) - tonumber(startingRow))
end

---Must return your bot from the script
return MyRandomBot
-- return MyDownBot