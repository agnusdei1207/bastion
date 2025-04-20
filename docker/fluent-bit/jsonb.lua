-- Pure Lua JSON 인코더
local function escape_str(s)
    return s:gsub('\\', '\\\\')
             :gsub('"', '\\"')
             :gsub('\n', '\\n')
             :gsub('\r', '\\r')
             :gsub('\t', '\\t')
end

local function encode_json(val)
    local t = type(val)
    if t == "nil" then
        return "null"
    elseif t == "boolean" or t == "number" then
        return tostring(val)
    elseif t == "string" then
        return '"' .. escape_str(val) .. '"'
    elseif t == "table" then
        local is_array = (#val > 0)
        local items = {}
        if is_array then
            for _, v in ipairs(val) do
                table.insert(items, encode_json(v))
            end
            return "[" .. table.concat(items, ",") .. "]"
        else
            for k, v in pairs(val) do
                local key = '"' .. escape_str(k) .. '"'
                local value = encode_json(v)
                table.insert(items, key .. ":" .. value)
            end
            return "{" .. table.concat(items, ",") .. "}"
        end
    else
        return '"<unsupported type>"'
    end
end

-- Fluent Bit Lua 필터 함수
function transform_record(tag, timestamp, record)
    local new_record = {}

    -- record의 모든 key-value 복사
    for k, v in pairs(record) do
        new_record[k] = v
    end

    -- 타임스탬프 ISO 8601 포맷 추가
    new_record["timestamp"] = os.date("%Y-%m-%dT%H:%M:%S%z", timestamp)

    -- JSON 직렬화 (에러 방지)
    local ok, jsonb = pcall(encode_json, record)
    if ok then
        new_record["log_data_jsonb"] = jsonb
    else
        new_record["log_data_jsonb"] = '{"error": "encoding failed"}'
    end

    -- alert 이벤트에 대해 추가 필드 설정
    if record["event_type"] == "alert" then
        new_record["alert"] = true
        if type(record["alert"]) == "table" then
            new_record["signature"] = record["alert"]["signature"] or ""
            new_record["severity"] = record["alert"]["severity"] or 0
        else
            new_record["signature"] = ""
            new_record["severity"] = 0
        end
    end

    return 1, timestamp, new_record
end
