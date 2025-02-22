#!/bin/python3

import numpy as np
import cv2 as cv
import sys

# Open the video
video_in = cv.VideoCapture(sys.argv[1])
if not video_in.isOpened():
    sys.exit(1)

video_shape = (int(video_in.get(cv.CAP_PROP_FRAME_WIDTH)), int(video_in.get(cv.CAP_PROP_FRAME_HEIGHT)))
video_fps = video_in.get(cv.CAP_PROP_FPS)
video_out = cv.VideoWriter(sys.argv[1] + ".avi", cv.VideoWriter_fourcc(*'FMP4'), video_fps, video_shape)
threshold = int(sys.argv[2]) if len(sys.argv) > 2 else 220

while video_in.isOpened():

    # Read frame
    ret, img = video_in.read()
    if not ret:
        break

    # Watermark is in the bottom corner right so limit ourselves to this area
    rows, cols, channels = img.shape
    print("Decoding frame...")
    rowP = 110
    colP = 400
    watermark_area = img[rows - rowP:-1, cols - colP:-1]

    # Convert to gray-scale and apply a threshold to create a mask of the watermark on this frame
    watermark_area = cv.cvtColor(watermark_area, cv.COLOR_BGR2GRAY)
    ret, watermark_mask = cv.threshold(watermark_area, threshold, 255, cv.THRESH_TOZERO)
    watermark_mask[0:60, 0:140] = np.zeros(shape=(60, 140), dtype=np.uint8)
    watermark_mask[87:110, 0:400] = np.zeros(shape=(22, 399), dtype=np.uint8)
    watermark_mask[0:110, -30:400] = np.zeros(shape=(109, 30), dtype=np.uint8)


    # Dilate the mask in order to not let any artifact around
    kernel = np.ones((3, 3), np.uint8)
    watermark_mask = cv.dilate(watermark_mask, kernel, iterations=1)

    #cv.imshow('dst', watermark_mask)
    #cv.waitKey(0)

    # Expand our watermark mask with black to fill the whole size of the frame
    mask = np.zeros(shape=(rows, cols))
    mask[rows - rowP:-1, cols - colP:-1] = watermark_mask

    # cv.imwrite("mask.png", mask)
    # mask = cv.imread("mask.png", 0)

    # Replace pixel of the watermark with neighbour colors
    img_without_watermark = cv.inpaint(img, mask.astype('uint8'), 6, cv.INPAINT_TELEA)
    video_out.write(img_without_watermark)
    # cv.imwrite("test.png", dst)
    # cv.imshow('dst',dst)
    # cv.waitKey(0)

video_in.release()
video_out.release()
cv.destroyAllWindows()
