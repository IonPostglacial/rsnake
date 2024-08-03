const canvas = document.getElementById("canvas");
const scoreDisplay = document.getElementById("score-display");
const up = document.getElementById("up");
const left = document.getElementById("left");
const down = document.getElementById("down");
const right = document.getElementById("right");
const cx = canvas.getContext("2d");
let stepPeriod = 300;

export function canvas_set_fill_style(hexColor) {
    var color = "#" + hexColor.toString(16).padStart(6, "0");
    cx.fillStyle = color;
}

export function canvas_fill_rect(x, y, width, height) {
    cx.fillRect(x, y, width, height);
}

export function canvas_fill() {
    cx.fill();
}

export function snake_score_changed(score) {
    scoreDisplay.textContent = score;
}

export function snake_step_period_updated(period) {
    stepPeriod = period;
}

export function snake_game_over() {
    window.alert("Game Over !");
    window.location.reload();
}

export function js_random(max) {
    return Math.random() * max;
}

const keys = {
    ArrowUp: 0,
    ArrowDown: 1,
    ArrowLeft: 2,
    ArrowRight: 3,
};

window.addEventListener("TrunkApplicationStarted", () => {
    let game = new window.wasmBindings.GameState();

    window.onkeydown = (e) => {
        e.stopPropagation();
        game.on_key_down(keys[e.code]);
    };

    up.onclick = () => game.on_key_down(keys.ArrowUp);
    down.onclick = () => game.on_key_down(keys.ArrowDown);
    left.onclick = () => game.on_key_down(keys.ArrowLeft);
    right.onclick = () => game.on_key_down(keys.ArrowRight);

    let lastUpdateTimestamp = -1;

    function step(timestamp) {
        if (lastUpdateTimestamp < 0) lastUpdateTimestamp = timestamp;
        const progress = timestamp - lastUpdateTimestamp;
        if (progress >= stepPeriod) {
            lastUpdateTimestamp = timestamp;
            game.step(progress);
        }
        window.requestAnimationFrame(step);
    }

    window.requestAnimationFrame(step);
});

