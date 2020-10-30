<template>
  <header
    ref="search"
    class="masthead d-flex w-50 ustify-content-center mx-auto p-3"
    style="height: 70px;"
  >
    <select
      class="form-control selectpicker"
      v-model="state.filter"
      data-live-search="true"
      title="Search tags"
    >
      <option v-for="tag in tags" :key="tag" :data-token="tag" :value="tag">
        {{ tag }}
      </option>
    </select>
    <button
      type="button"
      class="btn btn-primary ml-2 font-weight-bold"
      data-toggle="modal"
      data-target="#addCoub"
    >
      +
    </button>
    <div class="modal fade" id="addCoub" tabindex="-1">
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title" id="exampleModalLabel">Add Coub video</h5>
            <a
              class="close"
              data-dismiss="modal"
              aria-label="Close"
            >
              <span aria-hidden="true">&times;</span>
            </a>
          </div>
          <div class="modal-body">
            <form v-on:submit="addCoub">
              <div class="form-group">
                <label for="coubName">Coub name</label>
                <input
                  type="text"
                  class="form-control"
                  id="coubName"
                  ref="coubName"
                />
              </div>
            </form>
          </div>
          <div class="modal-footer">
            <button
              type="button"
              class="btn btn-secondary"
              data-dismiss="modal"
            >
              Close
            </button>
            <button type="button" class="btn btn-primary" v-on:click="addCoub">
              Add Coub
            </button>
          </div>
        </div>
      </div>
    </div>
  </header>

  <main role="main" class="inner d-flex">
    <Thumbnails :thumbnails="state.thumbnails" />
  </main>
</template>

<script lang="ts">
import { defineComponent, reactive } from "vue";
import Thumbnails, { Thumbnail } from "./components/Thumbnails.vue";
import * as _ from "lodash";
import $ from "jquery";
import "bootstrap-select";
import axios from "types-axios";

export default defineComponent({
  components: {
    Thumbnails
  },
  watch: {
    "state.filter": function(tag) {
      const vids = _.filter(
        this.thumbnailsData,
        (val: Thumbnail) =>
          val.tags.findIndex(val => _.includes(val, encodeURI(tag))) > -1
      );
      this.state.thumbnails = vids as never;
    }
  },
  setup() {
    const thumbnailsData = _.reverse((window as any).dataMock as Thumbnail[]);
    const tags = _.uniq(
      _.flatMap(thumbnailsData, (thumb: Thumbnail) => thumb.tags)
        .map(decodeURI)
        .filter((tag: string) => tag.length < 10)
    ).sort();

    const state = reactive({
      filter: "",
      thumbnails: thumbnailsData
    });

    return {
      state,
      tags,
      thumbnailsData
    };
  },
  mounted() {
    $(".selectpicker").selectpicker();
  },
  methods: {
    addCoub: function(e: UIEvent) {
      e.preventDefault();
      const coubName = (this.$refs.coubName as HTMLInputElement).value.trim();
      (this.$refs.coubName as HTMLInputElement).value = "Fetching...";
      axios
        .put("/api/video/" + coubName)
        .then(() => {
          (this.$refs.coubName as HTMLInputElement).value = "Success";
        })
        .catch(() => {
          console.log("failure");
          (this.$refs.coubName as HTMLInputElement).value = "Failure";
        });
    }
  }
});
</script>

<style scoped></style>
