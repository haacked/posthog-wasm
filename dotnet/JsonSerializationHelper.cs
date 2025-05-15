using System.Collections.ObjectModel;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace PostHog.Json;

internal static class JsonSerializerHelper
{
    static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        Converters =
        {
            new ReadOnlyCollectionJsonConverterFactory(),
            new ReadOnlyDictionaryJsonConverterFactory()
        }
    };

    public static string FormatJson(string json)
    {
        using var jsonDoc = JsonDocument.Parse(json);
        return JsonSerializer.Serialize(jsonDoc.RootElement, IndentedOptions);
    }

    static readonly JsonSerializerOptions IndentedOptions = new(Options)
    {
        WriteIndented = true
    };
    public static async Task<string> SerializeToCamelCaseJsonStringAsync<T>(T obj, bool writeIndented = false)
    {
        var stream = await SerializeToCamelCaseJsonStreamAsync(obj, writeIndented);
        stream.Position = 0;
        using var memoryStream = new MemoryStream();
        await stream.CopyToAsync(memoryStream);
        return Encoding.UTF8.GetString(memoryStream.ToArray());
    }

    static async Task<Stream> SerializeToCamelCaseJsonStreamAsync<T>(T obj, bool writeIndented = false)
    {
        var options = writeIndented ? IndentedOptions : Options;
        var stream = new MemoryStream();
        await JsonSerializer.SerializeAsync(stream, obj, options);
        return stream;
    }

    public static async Task<T?> DeserializeFromCamelCaseJsonStringAsync<T>(string json)
    {
        using var jsonStream = new MemoryStream(Encoding.UTF8.GetBytes(json));
        jsonStream.Position = 0;
        return await DeserializeFromCamelCaseJsonAsync<T>(jsonStream);
    }

    public static async Task<T?> DeserializeFromCamelCaseJsonAsync<T>(
        Stream jsonStream,
        CancellationToken cancellationToken = default) =>
        await JsonSerializer.DeserializeAsync<T>(jsonStream, Options, cancellationToken);
}

internal sealed class ReadOnlyCollectionJsonConverterFactory : JsonConverterFactory
{
    public override bool CanConvert(Type typeToConvert)
    {
        if (!typeToConvert.IsGenericType)
        {
            return false;
        }

        var genericTypeDefinition = typeToConvert.GetGenericTypeDefinition();
        return genericTypeDefinition == typeof(IReadOnlyCollection<>)
            || genericTypeDefinition == typeof(ReadOnlyCollection<>)
            || genericTypeDefinition == typeof(IReadOnlyList<>);
    }

    public override JsonConverter? CreateConverter(Type typeToConvert, JsonSerializerOptions options)
    {
        var elementType = typeToConvert.GetGenericArguments()[0];
        var converterType = typeof(ReadOnlyCollectionJsonConverterFactory<>).MakeGenericType(elementType);

        return (JsonConverter?)Activator.CreateInstance(converterType);
    }
}

internal sealed class ReadOnlyCollectionJsonConverterFactory<T> : JsonConverter<IEnumerable<T>>
{
    public override IEnumerable<T>? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        var list = JsonSerializer.Deserialize<List<T>>(ref reader, options);
        return list == null ? null : new ReadOnlyCollection<T>(list);
    }

    public override void Write(Utf8JsonWriter writer, IEnumerable<T> value, JsonSerializerOptions options)
        => JsonSerializer.Serialize(writer, value, options);
}

internal class ReadOnlyDictionaryJsonConverterFactory : JsonConverterFactory
{
    public override bool CanConvert(Type typeToConvert)
    {
        if (!typeToConvert.IsGenericType)
        {
            return false;
        }

        var genericTypeDefinition = typeToConvert.GetGenericTypeDefinition();
        return genericTypeDefinition == typeof(IReadOnlyDictionary<,>)
               || genericTypeDefinition == typeof(ReadOnlyDictionary<,>);
    }

    public override JsonConverter? CreateConverter(Type typeToConvert, JsonSerializerOptions options)
    {
        var keyType = typeToConvert.GetGenericArguments()[0];
        var valueType = typeToConvert.GetGenericArguments()[1];
        var converterType = typeof(ReadonlyDictionaryJsonConverter<,>).MakeGenericType(keyType, valueType);

        return (JsonConverter?)Activator.CreateInstance(converterType);
    }
}

public class ReadonlyDictionaryJsonConverter<TKey, TValue> : JsonConverter<IReadOnlyDictionary<TKey, TValue>> where TKey : notnull
{
    public override IReadOnlyDictionary<TKey, TValue>? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        var dictionary = JsonSerializer.Deserialize<Dictionary<TKey, TValue>>(ref reader, options);
        return dictionary == null ? null : new ReadOnlyDictionary<TKey, TValue>(dictionary);
    }

    public override void Write(Utf8JsonWriter writer, IReadOnlyDictionary<TKey, TValue> value, JsonSerializerOptions options)
    {
        JsonSerializer.Serialize(writer, value, options);
    }
}