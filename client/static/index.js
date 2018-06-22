const agar = import("./agar");
const ext = import("./ext");

ws = new WebSocket("ws://127.0.0.1:6969");

function get_size() {
    var w = window.innerWidth;
    var h = window.innerHeight - 20;

    return [w, h];
}

let canvas = document.getElementById("draw");
let ctx = canvas.getContext("2d");

var width = get_size()[0];
var height = get_size()[1];

canvas.width = width;
canvas.height = height;

ctx.textAlign = "center";
ctx.font = "5px monospace";

let amount = 0;

agar.then(module => {
        ws.onmessage = msg => {
            module.recv_ws_message(msg.data);
        }
        module.start(width, height);

        document.body.addEventListener("mousemove", event => {
            module.mouse_moved(event.x, event.y);
        });
        setInterval(module.tick, 1000 / 60);
        setInterval(module.redraw, 1000 * 10);
        setInterval(() => {
            if (get_size()[0] !== width || get_size()[1] !== height) {
                width = get_size()[0];
                height = get_size()[1];

                canvas.width = width;
                canvas.height = height;

                ctx.textAlign = "center";
                ctx.font = "5px monospace";

                module.resize(width, height);
            }
        }, 50);

    })

