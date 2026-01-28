#pragma once
#include <atomic>
#include <array>
#include <cstdint>
#include <mutex>
#include <vector>

#include "flutter/texture_registrar.h"

class NesiumTexture
{
public:
    NesiumTexture(int width, int height)
        : width_(width), height_(height), stride_(width * 4), latest_(0)
    {
        buffers_[0].resize(static_cast<size_t>(stride_) * height_);
        buffers_[1].resize(static_cast<size_t>(stride_) * height_);
        pixel_buffer_.width = width_;
        pixel_buffer_.height = height_;
        pixel_buffer_.buffer = buffers_[0].data();
    }

    std::pair<uint8_t *, int> acquireWritableBuffer()
    {
        int cur = latest_.load(std::memory_order_acquire);
        int next = 1 - cur;
        return {buffers_[next].data(), next};
    }

    void commitLatestReady(int index)
    {
        latest_.store(index, std::memory_order_release);
    }

    const FlutterDesktopPixelBuffer *CopyPixelBuffer(size_t, size_t)
    {
        std::lock_guard<std::mutex> lk(mu_);
        int idx = latest_.load(std::memory_order_acquire);
        pixel_buffer_.buffer = buffers_[idx].data();
        pixel_buffer_.width = width_;
        pixel_buffer_.height = height_;
        return &pixel_buffer_;
    }

    void Resize(int width, int height)
    {
        std::lock_guard<std::mutex> lk(mu_);
        if (width == width_ && height == height_) {
            return;
        }
        width_ = width;
        height_ = height;
        stride_ = width_ * 4;
        buffers_[0].assign(static_cast<size_t>(stride_) * height_, 0);
        buffers_[1].assign(static_cast<size_t>(stride_) * height_, 0);
        latest_.store(0, std::memory_order_release);
        pixel_buffer_.width = width_;
        pixel_buffer_.height = height_;
        pixel_buffer_.buffer = buffers_[0].data();
    }

    int width() const { return width_; }
    int stride() const { return stride_; }
    int height() const { return height_; }

private:
    int width_;
    int height_;
    int stride_;
    std::array<std::vector<uint8_t>, 2> buffers_;
    std::atomic<int> latest_;
    FlutterDesktopPixelBuffer pixel_buffer_{};
    std::mutex mu_;
};
