#include "nesium_texture_plugin.h"

#include <windows.h>
#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <memory>
#include <mutex>
#include <thread>

#include "flutter/method_channel.h"
#include "flutter/plugin_registrar_windows.h"
#include "flutter/standard_method_codec.h"
#include "flutter/texture_registrar.h"

#include "nesium_texture.h"

namespace
{

    using NesiumFrameReadyCallback = void (*)(uint32_t, uint32_t, uint32_t, uint32_t, void *);

    struct RustApi
    {
        HMODULE dll = nullptr;

        void (*runtime_start)() = nullptr;
        void (*set_frame_ready_callback)(NesiumFrameReadyCallback cb, void *user) = nullptr;
        void (*copy_frame)(uint32_t bufferIndex, uint8_t *dst, uint32_t dstPitch, uint32_t dstHeight) = nullptr;

        bool Load(const wchar_t *dll_name)
        {
            dll = ::LoadLibraryW(dll_name);
            if (!dll)
                return false;

            runtime_start = reinterpret_cast<decltype(runtime_start)>(::GetProcAddress(dll, "nesium_runtime_start"));
            set_frame_ready_callback = reinterpret_cast<decltype(set_frame_ready_callback)>(::GetProcAddress(dll, "nesium_set_frame_ready_callback"));
            copy_frame = reinterpret_cast<decltype(copy_frame)>(::GetProcAddress(dll, "nesium_copy_frame"));

            return runtime_start && set_frame_ready_callback && copy_frame;
        }
    };

    class NesiumTexturePlugin : public flutter::Plugin
    {
    public:
        explicit NesiumTexturePlugin(flutter::PluginRegistrarWindows *registrar)
            : registrar_(registrar), texture_registrar_(registrar->texture_registrar())
        {
            channel_ = std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
                registrar_->messenger(), "nesium", &flutter::StandardMethodCodec::GetInstance());

            channel_->SetMethodCallHandler(
                [this](const auto &call, auto result)
                { HandleMethodCall(call, std::move(result)); });

            worker_ = std::thread([this]
                                  { CopyWorkerMain(); });
        }

        ~NesiumTexturePlugin() override
        {
            shutting_down_.store(true, std::memory_order_release);
            {
                std::lock_guard<std::mutex> lk(mu_);
                cv_.notify_all();
            }
            if (worker_.joinable())
                worker_.join();
        }

    private:
        void HandleMethodCall(
            const flutter::MethodCall<flutter::EncodableValue> &call,
            std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result)
        {
            if (call.method_name() == "createNesTexture")
            {
                CreateNesTexture(std::move(result));
                return;
            }
            result->NotImplemented();
        }

        void CreateNesTexture(std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result)
        {
            if (texture_id_.load(std::memory_order_acquire) >= 0)
            {
                result->Success(flutter::EncodableValue(texture_id_.load()));
                return;
            }

            if (!rust_.Load(L"nesium_flutter.dll"))
            {
                result->Error("dlopen_failed", "Failed to load nesium_flutter.dll (place it next to Runner.exe).");
                return;
            }

            const int width = 256;
            const int height = 240;
            texture_ = std::make_unique<NesiumTexture>(width, height);

            texture_variant_ = std::make_unique<flutter::TextureVariant>(
                flutter::PixelBufferTexture([this](size_t w, size_t h) -> const FlutterDesktopPixelBuffer *
                                            { return texture_ ? texture_->CopyPixelBuffer(w, h) : nullptr; }));

            const int64_t id = texture_registrar_->RegisterTexture(texture_variant_.get());
            texture_id_.store(id, std::memory_order_release);

            rust_.set_frame_ready_callback(&NesiumTexturePlugin::OnFrameReadyThunk, this);
            rust_.runtime_start();

            result->Success(flutter::EncodableValue(id));
        }

        static void OnFrameReadyThunk(uint32_t bufferIndex, uint32_t width, uint32_t height, uint32_t pitch, void *user)
        {
            static_cast<NesiumTexturePlugin *>(user)->OnFrameReady(bufferIndex, width, height, pitch);
        }

        void OnFrameReady(uint32_t bufferIndex, uint32_t, uint32_t, uint32_t)
        {
            pending_index_.store(bufferIndex, std::memory_order_release);

            bool expected = false;
            if (!copy_scheduled_.compare_exchange_strong(expected, true, std::memory_order_acq_rel))
            {
                return;
            }

            std::lock_guard<std::mutex> lk(mu_);
            cv_.notify_one();
        }

        void CopyWorkerMain()
        {
            const uint32_t empty = 0xFFFFFFFFu;

            while (!shutting_down_.load(std::memory_order_acquire))
            {
                {
                    std::unique_lock<std::mutex> lk(mu_);
                    cv_.wait(lk, [this]
                             { return shutting_down_.load(std::memory_order_acquire) ||
                                      copy_scheduled_.load(std::memory_order_acquire); });
                }
                if (shutting_down_.load(std::memory_order_acquire))
                    break;

                const uint32_t idx = pending_index_.exchange(empty, std::memory_order_acq_rel);
                copy_scheduled_.store(false, std::memory_order_release);

                auto *tex = texture_.get();
                const int64_t tid = texture_id_.load(std::memory_order_acquire);
                if (!tex || tid < 0 || idx == empty)
                    continue;

                auto [dst, write_index] = tex->acquireWritableBuffer();
                rust_.copy_frame(idx, dst, static_cast<uint32_t>(tex->stride()), static_cast<uint32_t>(tex->height()));
                tex->commitLatestReady(write_index);

                texture_registrar_->MarkTextureFrameAvailable(tid);

                if (pending_index_.load(std::memory_order_acquire) != empty)
                {
                    bool expected = false;
                    if (copy_scheduled_.compare_exchange_strong(expected, true, std::memory_order_acq_rel))
                    {
                        std::lock_guard<std::mutex> lk(mu_);
                        cv_.notify_one();
                    }
                }
            }
        }

    private:
        flutter::PluginRegistrarWindows *registrar_;
        flutter::TextureRegistrar *texture_registrar_;
        std::unique_ptr<flutter::MethodChannel<flutter::EncodableValue>> channel_;

        RustApi rust_;

        std::unique_ptr<NesiumTexture> texture_;
        std::unique_ptr<flutter::TextureVariant> texture_variant_;

        std::atomic<int64_t> texture_id_{-1};
        std::atomic<uint32_t> pending_index_{0xFFFFFFFFu};
        std::atomic<bool> copy_scheduled_{false};
        std::atomic<bool> shutting_down_{false};

        std::mutex mu_;
        std::condition_variable cv_;
        std::thread worker_;
    };

} // namespace

void NesiumTexturePluginRegisterWithRegistrar(
    flutter::PluginRegistrarWindows *registrar)
{
    auto plugin = std::make_unique<NesiumTexturePlugin>(registrar);
    registrar->AddPlugin(std::move(plugin));
}