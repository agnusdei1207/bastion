local dkjson = require("dkjson")

function transform_record(tag, timestamp, record)
    -- 필요한 필드 추출
    local new_record = {}
    
    -- 타임스탬프 포맷 변환 (ISO 8601 형식)
    new_record["timestamp"] = os.date("%Y-%m-%dT%H:%M:%S%z", timestamp)
    
    -- 필요한 필드 복사
    new_record["src_ip"] = record["src_ip"]
    new_record["dest_ip"] = record["dest_ip"]
    new_record["action"] = record["action"]
    new_record["event_type"] = record["event_type"]
    
    -- 전체 레코드를 JSON 문자열로 변환하여 저장 (Fluentd에서 하던 방식과 유사)
    new_record["log_data_jsonb"] = dkjson.encode(record)
    
    -- 알림, 경고 등 중요 이벤트 처리
    if record["event_type"] == "alert" then
        new_record["alert"] = true
        new_record["signature"] = record["alert"] and record["alert"]["signature"] or ""
        new_record["severity"] = record["alert"] and record["alert"]["severity"] or 0
    end
    
    -- 기존 fluentd 설정과 유사하게 필드 이름 매핑
    if record["flow_id"] then
        new_record["flow_id"] = record["flow_id"]
    end
    
    if record["proto"] then
        new_record["protocol"] = record["proto"]
    end
    
    -- 원본 데이터가 필요한 경우를 위해 원본 필드도 보존
    for k, v in pairs(record) do
        if new_record[k] == nil then
            new_record[k] = v
        end
    end
    
    return 1, timestamp, new_record
end