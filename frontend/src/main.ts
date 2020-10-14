import { createApp } from "vue";
import App from "./App.vue";

import "jquery";
import "bootstrap";
import "popper.js";

import axios from "axios";

axios.get("/api/videos").then((response) => {
    console.log(response);
    (window as any).dataMock = response.data;
}).catch((error) => {
    console.error(error);
}).then(() => {
    createApp(App).mount("#app");
});
