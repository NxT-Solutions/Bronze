import Foundation
import NaturalLanguage

#if canImport(FoundationModels)
import FoundationModels
#endif

private let titleMaxChars = 72
private let foundationWaitSeconds: TimeInterval = 8

@_silgen_name("bronze_native_item_title")
public func bronze_native_item_title(
    _ body: bronze_native_utf8_view,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>?
) -> UInt32 {
    let status = bronze_native_validate_utf8(body)
    if status != BRONZE_STATUS_OK {
        return status
    }
    guard let out else {
        return BRONZE_STATUS_NOT_FOUND
    }
    guard let text = utf8Body(body) else {
        return BRONZE_STATUS_INVALID_UTF8
    }
    if Thread.isMainThread {
        // C ABI is sync; wait off the main thread (model work must not run there).
        let box = StatusBox()
        let outBox = OutBox(out)
        let lock = DispatchSemaphore(value: 0)
        DispatchQueue.global(qos: .userInitiated).async {
            box.value = resolveAndCopyTitle(text, outBox.ptr)
            lock.signal()
        }
        _ = lock.wait(timeout: .now() + foundationWaitSeconds + 1)
        return box.value
    }
    return resolveAndCopyTitle(text, out)
}

private func resolveAndCopyTitle(
    _ text: String,
    _ out: UnsafeMutablePointer<bronze_native_utf8_view>
) -> UInt32 {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        return BRONZE_STATUS_DEGRADED
    }
    guard let title = resolveTitle(trimmed), !title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
        return BRONZE_STATUS_DEGRADED
    }
    let bytes = Array(title.utf8)
    return bytes.withUnsafeBufferPointer { buf in
        let src = bronze_native_utf8_view(ptr: buf.baseAddress, len: UInt64(bytes.count))
        return bronze_native_utf8_owned_copy(src, out)
    }
}

private func utf8Body(_ view: bronze_native_utf8_view) -> String? {
    if view.ptr == nil {
        return view.len == 0 ? "" : nil
    }
    if view.len > UInt64(Int.max) {
        return nil
    }
    let count = Int(view.len)
    let buffer = UnsafeBufferPointer(start: view.ptr, count: count)
    let bytes = Array(buffer)
    let decoded = String(decoding: bytes, as: UTF8.self)
    let reencoded = Array(decoded.utf8)
    guard reencoded == bytes else {
        return nil
    }
    return decoded
}

private func resolveTitle(_ text: String) -> String? {
    if let generated = foundationTitle(text) {
        let clamped = clampTitle(generated)
        if !clamped.isEmpty {
            return clamped
        }
    }
    if #available(macOS 11.0, *) {
        if let extractive = extractiveTitle(from: text), !extractive.isEmpty {
            return extractive
        }
    }
    return nil
}

private func foundationTitle(_ text: String) -> String? {
#if canImport(FoundationModels)
    if #available(macOS 26.0, *) {
        return waitForFoundationTitle(text)
    }
#endif
    return nil
}

#if canImport(FoundationModels)
@available(macOS 26.0, *)
private func waitForFoundationTitle(_ text: String) -> String? {
    let model = SystemLanguageModel.default
    guard model.isAvailable else {
        return nil
    }
    let box = TitleBox()
    let lock = DispatchSemaphore(value: 0)
    let sample = String(text.prefix(8000))
    Task.detached {
        do {
            let instructions: String = """
            Produce a short title (max 12 words) that summarizes the captured text.
            Reply with the title only.
            Do not use the first sentence unless that sentence is actually the summary.
            """
            let session = LanguageModelSession(model: model, instructions: instructions)
            let response = try await session.respond(to: sample)
            let raw = response.content.trimmingCharacters(in: .whitespacesAndNewlines)
            if !raw.isEmpty {
                box.value = raw
            }
        } catch {
            box.value = nil
        }
        lock.signal()
    }
    // Give up after 8s; the detached task is not cancelled.
    if lock.wait(timeout: .now() + foundationWaitSeconds) == .timedOut {
        return nil
    }
    return box.value
}
#endif

@available(macOS 11.0, *)
func extractiveTitle(from text: String) -> String? {
    let sentences = sentenceTokens(text)
    guard !sentences.isEmpty else {
        return nil
    }
    let recognizer = NLLanguageRecognizer()
    recognizer.processString(text)
    let language = recognizer.dominantLanguage ?? .english
    guard let embedding = NLEmbedding.sentenceEmbedding(for: language)
        ?? NLEmbedding.sentenceEmbedding(for: .english)
    else {
        return nil
    }
    var kept: [String] = []
    var vectors: [[Double]] = []
    for sentence in sentences {
        if let vector = embedding.vector(for: sentence) {
            kept.append(sentence)
            vectors.append(vector)
        }
    }
    guard let picked = nearestToCentroid(sentences: kept, vectors: vectors) else {
        return nil
    }
    return clampTitle(picked)
}

func nearestToCentroid(sentences: [String], vectors: [[Double]]) -> String? {
    guard sentences.count == vectors.count, !vectors.isEmpty else {
        return nil
    }
    if sentences.count == 1 {
        return sentences[0]
    }
    let dim = vectors[0].count
    guard dim > 0, vectors.allSatisfy({ $0.count == dim }) else {
        return nil
    }
    var centroid = Array(repeating: 0.0, count: dim)
    for vector in vectors {
        for (idx, value) in vector.enumerated() {
            centroid[idx] += value
        }
    }
    let scale = 1.0 / Double(vectors.count)
    for idx in centroid.indices {
        centroid[idx] *= scale
    }
    var bestIdx = 0
    var bestDist = Double.greatestFiniteMagnitude
    for (idx, vector) in vectors.enumerated() {
        var dist = 0.0
        for (a, b) in zip(vector, centroid) {
            let delta = a - b
            dist += delta * delta
        }
        if dist < bestDist {
            bestDist = dist
            bestIdx = idx
        }
    }
    return sentences[bestIdx]
}

func clampTitle(_ text: String, maxChars: Int = titleMaxChars) -> String {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    if trimmed.count <= maxChars {
        return trimmed
    }
    let end = trimmed.index(trimmed.startIndex, offsetBy: maxChars - 1)
    return String(trimmed[..<end]) + "…"
}

@available(macOS 11.0, *)
private func sentenceTokens(_ text: String) -> [String] {
    let tokenizer = NLTokenizer(unit: .sentence)
    tokenizer.string = text
    var sentences: [String] = []
    tokenizer.enumerateTokens(in: text.startIndex..<text.endIndex) { range, _ in
        let piece = text[range].trimmingCharacters(in: .whitespacesAndNewlines)
        if !piece.isEmpty {
            sentences.append(piece)
        }
        return true
    }
    if sentences.isEmpty {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmed.isEmpty {
            sentences.append(trimmed)
        }
    }
    return sentences
}

private final class TitleBox: @unchecked Sendable {
    var value: String?
}

private final class StatusBox: @unchecked Sendable {
    var value: UInt32 = BRONZE_STATUS_DEGRADED
}

private final class OutBox: @unchecked Sendable {
    let ptr: UnsafeMutablePointer<bronze_native_utf8_view>
    init(_ ptr: UnsafeMutablePointer<bronze_native_utf8_view>) {
        self.ptr = ptr
    }
}
