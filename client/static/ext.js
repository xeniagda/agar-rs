let canvas = document.getElementById("draw");
let ctx = canvas.getContext("2d");

function get_size() {
    var w = window.innerWidth;
    var h = window.innerHeight;

    return [w, h];
}

export function put_char_3(x, y, ch, fr, fg, fb) {
    ctx.fillStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.fillText(String.fromCharCode(ch), x, y);
}

export function put_bg_3(fr, fg, fb) {
    ctx.fillStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
}

export function put_circle_3(x, y, r, fr, fg, fb, or, og, ob) {
    if (x + r < 0 ||
        y + r < 0 ||
        x - r > get_size()[0] ||
        y - r > get_size()[1] )
    {
        return;
    }

    ctx.fillStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.strokeStyle = `rgb(${or & 255},${og & 255},${ob & 255})`;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
}

export function put_line_3(x1, y1, x2, y2, r, fr, fg, fb) {
    ctx.strokeStyle = `rgb(${fr & 255},${fg & 255},${fb & 255})`;
    ctx.lineWidth = r;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.closePath();
    ctx.stroke();
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
    ws.send(new Uint8Array(data).buffer);
}
