const agar = import("./agar");
const ext = import("./ext");


function get_size() {
    var w = window.innerWidth;
    var h = window.innerHeight;

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
        ws = new WebSocket("ws://" + window.location.hostname + ":6969");
        ws.binaryType = "arraybuffer";

        ws.onmessage = msg => {
            module.recv_ws(Array.from(new Uint8Array(msg.data)));
        }
        module.start(width, height);

        document.body.addEventListener("mousemove", event => {
            module.mouse_moved(event.x, event.y);
        });

        document.body.addEventListener("touchmove", event => {
            window.scrollTo(0, 0);
            event.preventDefault();

            let touch = event.touches[0];

            if (!isNaN(touch.clientX) && !isNaN(touch.clientY)) {
                module.mouse_moved(touch.clientX, touch.clientY);
            }
        });

        document.body.addEventListener("wheel", event => {
            module.scroll(event.deltaY);
        });

        function ticker() {
            let d = new Date();
            module.tick(d.getTime() / 1000);
            requestAnimationFrame(ticker);
        }
        requestAnimationFrame(ticker);

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

