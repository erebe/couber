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
        data-bs-toggle="tooltip"
        v-bind:data-name="thumbnail.name"
        v-bind:title="decodeURI(thumbnail.tags.toString())"
        v-bind:poster="thumbnail.thumbnail"
        v-on:auxclick="addTags"
      >
        <source v-bind:src="thumbnail.url" type="video/mp4" />
      </video>
    </div>

    <div
      class="modal fade"
      id="addTagsModal"
      ref="addTagsModal"
      tabindex="-1"
    >
      <div class="modal-dialog">
        <div class="modal-content">
          <div class="modal-header">
            <h5 class="modal-title">Add tags</h5>
            <button
              type="button"
              class="btn-close"
              data-bs-dismiss="modal"
              aria-label="Close"
            ></button>
          </div>
          <div class="modal-body">
            <form>
              <div class="mb-3">
                <label for="videoTagsInput" class="form-label">Tags</label>
                <textarea
                  class="form-control"
                  id="videoTagsInput"
                  ref="videoTagsInput"
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
            <button
              type="button"
              class="btn btn-primary"
              v-on:click="addTagsSubmit"
            >
              Add tags
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, type PropType } from "vue";
import * as _ from "lodash";
import { Modal } from "bootstrap";
import axios from "axios";

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

  const observer = new IntersectionObserver((entries) => {
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
  data: function() {
    return {
      selectedVideo: {} as Thumbnail
    };
  },
  methods: {
    addTags: function(event: MouseEvent) {
      const videoName = (event.target as HTMLVideoElement).dataset.name;
      const video = this.thumbnails.find(el => el.name == videoName);
      if (_.isNil(video)) return;
      this.selectedVideo = video;

      const tagsInput = this.$refs.videoTagsInput as HTMLTextAreaElement;
      tagsInput.value = _.map(video.tags, tag => decodeURI(tag)).toString();
      const modal = new Modal(this.$refs.addTagsModal as HTMLElement);
      modal.show();
    },
    addTagsSubmit: function(event: MouseEvent) {
      event.preventDefault();
      const tagsInput = this.$refs.videoTagsInput as HTMLTextAreaElement;
      const tags = _.map(tagsInput.value.split(","), tag =>
        encodeURI(tag.trim())
      );
      const tagsToAdd = _.difference(tags, _.values(this.selectedVideo.tags));
      axios
        .put(
          "/api/video/" + this.selectedVideo.name + "/tags",
          JSON.stringify(tagsToAdd),
          { headers: { "content-type": "application/json" } }
        )
        .then(() => {
          this.selectedVideo.tags = tags;
          const modal = Modal.getInstance(this.$refs.addTagsModal as HTMLElement);
          modal?.hide();
        })
        .catch(err => console.error(err));
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

textarea {
  height: 300px;
}
</style>