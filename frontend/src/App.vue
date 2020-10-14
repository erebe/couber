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


export default defineComponent({
  components: {
    Thumbnails
  },
  watch: {
    "state.filter": function(tag, oldTag) {
      const vids = _.filter(
        this.thumbnailsData,
        (val: Thumbnail) =>
          val.tags.findIndex((val, ix) => _.includes(val, encodeURI(tag))) > -1
      );
      this.state.thumbnails = vids as never;
    }
  },
  setup() {
    const thumbnailsData = (window as any).dataMock as Thumbnail[];
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
    $('.selectpicker').selectpicker();
  }
});
</script>
