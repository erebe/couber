<template>
  <div
    id="videosContainer"
    ref="videos"
    class="d-flex flex-row cover flex-wrap justify-content-center"
  >
    <div
      class="thumbnail"
      v-for="thumbnail in thumbnails"
      v-bind:key="thumbnail.name"
    >
      <video
        controls
        preload="none"
        data-toggle="tooltip"
        v-bind:title="decodeURI(thumbnail.tags)"
      >
        <source v-bind:src="thumbnail.url" type="video/mp4" />
      </video>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";

export interface Thumbnail {
  name: string;
  url: string;
  tags: string[];
}

const respondToVisibility = function(element: HTMLElement, callback: any) {
  const options = {
    root: null
  };

  const observer = new IntersectionObserver((entries, observer) => {
    entries.forEach(entry => {
      callback(entry, entry.intersectionRatio > 0);
    });
  }, options);

  observer.observe(element);
};

export default defineComponent({
  props: {
    thumbnails: {
      type: Object as PropType<Thumbnail[]>,
      required: true
    }
  },
  mounted() {
    this.$forceUpdate();
  },
  updated() {
    //TODO: use refs when https://github.com/vuejs/vue-next/issues/1166 is fixes
    const videos = (this.$refs.videos as HTMLDivElement).getElementsByTagName(
      "video"
    );
    for (let i = 0; i < videos.length; i++) {
      respondToVisibility(
        videos.item(i)!,
        (entry: IntersectionObserverEntry, visible: boolean) => {
          const el = entry.target as HTMLVideoElement;
          if (visible && el.preload != "metadata") {
            el.preload = "metadata";
            el.style.visibility = "visible";
          }
        }
      );
    }
  }
});
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.thumbnail {
  /* border-block: solid;
  border-style: dotted; */
  margin: 10px;
  width: 400px;
  height: 300px;
}

video {
  object-fit: contain;
  visibility: hidden;
  width: 100%;
  height: 100%;
}
</style>
