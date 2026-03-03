<template>
  <header
    ref="search"
    class="masthead d-flex w-50 justify-content-center mx-auto p-3"
    style="height: 70px;"
  >
    <input
      class="form-control"
      list="tags-list"
      v-model="state.filter"
      placeholder="Search tags"
      @change="onFilterChange"
    />
    <datalist id="tags-list">
      <option v-for="tag in tags" :key="tag" :value="tag">{{ tag }}</option>
    </datalist>
    <button
      type="button"
      class="btn btn-primary ms-2 fw-bold"
      data-bs-toggle="modal"
      data-bs-target="#addCoub"
    >
      +
    </button>
    <div class="modal fade" id="addCoub" tabindex="-1">
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Add video</h5>
            <button
              type="button"
              class="btn-close"
              data-bs-dismiss="modal"
              aria-label="Close"
            ></button>
          </div>
          <div class="modal-body">
            <form v-on:submit="addCoub">
              <div class="mb-3">
                <label for="coubName" class="form-label">Video url</label>
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
              data-bs-dismiss="modal"
            >
              Close
            </button>
            <button type="button" class="btn btn-primary" v-on:click="addCoub">
              Add Video
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
import Thumbnails, { type Thumbnail } from "./components/Thumbnails.vue";
import * as _ from "lodash";
import axios from "axios";

export default defineComponent({
  components: {
    Thumbnails
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
  methods: {
    onFilterChange() {
      const tag = this.state.filter;
      if (!tag) {
        this.state.thumbnails = this.thumbnailsData as never;
        return;
      }
      const vids = _.filter(
        this.thumbnailsData,
        (val: Thumbnail) =>
          val.tags.findIndex(t => _.includes(t, encodeURI(tag))) > -1
      );
      this.state.thumbnails = vids as never;
    },
    addCoub: function(e: Event) {
      e.preventDefault();

      const coubName = (this.$refs.coubName as HTMLInputElement).value.trim();
      (this.$refs.coubName as HTMLInputElement).value = "Fetching...";
      axios
        .put("/api/video", { url: coubName })
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