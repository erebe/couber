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
        v-bind:poster="thumbnail.thumbnail"
      >
        <source v-bind:src="thumbnail.url" type="video/mp4" />
      </video>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import * as _ from "lodash";

export interface Thumbnail {
  name: string;
  url: string;
  tags: string[];
  thumbnail: string;
  original: string;
}

const loadMoreVideos = function(container: HTMLDivElement, loadNbMore: number) {
  const videosContainers = _.values(
    container.getElementsByClassName("thumbnail")
  ) as HTMLDivElement[];
  const videosToMakeVisible = _.take(
    _.dropWhile(
      videosContainers,
      (video: HTMLDivElement) => video.style.display == "block"
    ),
    loadNbMore
  );

  _.forEach(
    videosToMakeVisible,
    videosContainer => (videosContainer.style.display = "block")
  );
};
const loadMoreVideosThrottled = _.throttle(loadMoreVideos, 100, {
  trailing: true,
  leading: false
});

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
    const renderVisible = (videoEl: HTMLVideoElement) => {
      if (videoEl.style.visibility != "visible") {
        videoEl.style.visibility = "visible";
        videoEl.parentElement!.style.display = "block";
      }
    };

    //TODO: use refs when https://github.com/vuejs/vue-next/issues/1166 is fixes
    const videosContainer = this.$refs.videos as HTMLDivElement;
    _.values(videosContainer.getElementsByTagName("video")).forEach(video => {
      respondToVisibility(
        video,
        (entry: IntersectionObserverEntry, visible: boolean) => {
          if (visible) {
            renderVisible(entry.target as HTMLVideoElement);
            loadMoreVideosThrottled(videosContainer, 10);
          }
        }
      );
    });
    loadMoreVideos(videosContainer, 10);
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
  display: none;
}

video {
  object-fit: contain;
  visibility: hidden;
  width: 100%;
  height: 100%;
}
</style>
