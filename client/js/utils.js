
// Start a new game
// mode: 'playerVsBot' | 'botVsBot'
// botName string
async function startGame(mode, botName) {
    var body = {
        mode,
        botName
    };
    console.log("input", JSON.stringify(body));
    var response = await fetch('/new_game', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
    });

    console.log("received response");

    if (response.ok) {
        const data = await response.json();
        const game_id = data.game_id;

        if (mode === 'playerVsBot') {
            window.location.href = `/game/${game_id}`;
        } else if (mode === 'botVsBot') {
            console.log(`/spectate/${game_id}`);
            window.location.href = `/spectate/${game_id}`;
        }
    } else {
        console.error('Failed to fetch:', response.statusText);
    }
}


function startBotGame(botName) {
    startGame('botVsBot', botName)
}

function startPlayerGame(botName) {
    startGame('playerVsBot', botName)
}