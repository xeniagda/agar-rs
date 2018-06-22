let canvas = document.getElementById("draw");
let ctx = canvas.getContext("2d");

export function put_char_3(x, y, ch, fr, fg, fb) {
    ctx.fillStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.fillText(String.fromCharCode(ch), x, y);
}

export function put_circle_3(x, y, r, fr, fg, fb) {
    ctx.fillStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.closePath();
    ctx.fill();
}

export function clear() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
}

export function log(text) {
    console.log(text)
}

export function rand() {
    return Math.random();
}

export function atan2(y, x) {
    return Math.atan2(y, x);
}

export function ws_send(data) {
    ws.send(data);
}
