import axios from "axios";
import {ApiResponse} from "./apiResponse";

export interface HealthCheck {
    id: number,
    create_time: number,
    request_time: number,
    node_name: string,
    status_code: number,
    status_des: string,
    delay_ms: number,
}

/**
 * 查询健康检查数据
 */
const list_health_check = (body: {start_time: number, end_time: number}) => {
    return axios.post<ApiResponse<HealthCheck[]>>("/api/proxy/list_health_check", body);
};

export default {
    list_health_check
}