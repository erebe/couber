import { createApp } from "vue";
import App from "./App.vue";

import "jquery";
import "bootstrap";
import "popper.js";
import axios from "axios";

//import "./debug/dataMock";

axios
  .get("/api/videos")
  .then(response => {
    (window as any).dataMock = response.data;
  })
  .catch(error => {
    console.error(error);
  })
  .then(() => {
    createApp(App).mount("#app");
  });
