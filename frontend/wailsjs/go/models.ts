export namespace app_services {
	
	export class ApiResponse__tulsu_pe_timetable_backend_config_Config_ {
	    data?: config.Config;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_config_Config_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], config.Config);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse__tulsu_pe_timetable_backend_storage_Direction_ {
	    data?: storage.Direction;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_storage_Direction_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Direction);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse__tulsu_pe_timetable_backend_storage_Faculty_ {
	    data?: storage.Faculty;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_storage_Faculty_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Faculty);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse__tulsu_pe_timetable_backend_storage_Lesson_ {
	    data?: storage.Lesson;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_storage_Lesson_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Lesson);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse__tulsu_pe_timetable_backend_storage_Semester_ {
	    data?: storage.Semester;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_storage_Semester_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Semester);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse__tulsu_pe_timetable_backend_storage_Teacher_ {
	    data?: storage.Teacher;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse__tulsu_pe_timetable_backend_storage_Teacher_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Teacher);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse___tulsu_pe_timetable_backend_storage_Direction_ {
	    data: storage.Direction[];
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse___tulsu_pe_timetable_backend_storage_Direction_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Direction);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse___tulsu_pe_timetable_backend_storage_Faculty_ {
	    data: storage.Faculty[];
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse___tulsu_pe_timetable_backend_storage_Faculty_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Faculty);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse___tulsu_pe_timetable_backend_storage_Lesson_ {
	    data: storage.Lesson[];
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse___tulsu_pe_timetable_backend_storage_Lesson_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Lesson);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse___tulsu_pe_timetable_backend_storage_Semester_ {
	    data: storage.Semester[];
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse___tulsu_pe_timetable_backend_storage_Semester_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Semester);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse___tulsu_pe_timetable_backend_storage_Teacher_ {
	    data: storage.Teacher[];
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse___tulsu_pe_timetable_backend_storage_Teacher_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = this.convertValues(source["data"], storage.Teacher);
	        this.error = source["error"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class ApiResponse_bool_ {
	    data: boolean;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse_bool_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = source["data"];
	        this.error = source["error"];
	    }
	}
	export class ApiResponse_string_ {
	    data: string;
	    error: string;
	
	    static createFrom(source: any = {}) {
	        return new ApiResponse_string_(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.data = source["data"];
	        this.error = source["error"];
	    }
	}

}

export namespace config {
	
	export class Config {
	    dbPath: string;
	    theme: string;
	
	    static createFrom(source: any = {}) {
	        return new Config(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.dbPath = source["dbPath"];
	        this.theme = source["theme"];
	    }
	}

}

export namespace storage {
	
	export class BindTeacherToSemesterRequest {
	    semester_id: number;
	    teacher_id: number;
	
	    static createFrom(source: any = {}) {
	        return new BindTeacherToSemesterRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.semester_id = source["semester_id"];
	        this.teacher_id = source["teacher_id"];
	    }
	}
	export class CreateDirectionRequest {
	    name: string;
	
	    static createFrom(source: any = {}) {
	        return new CreateDirectionRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	    }
	}
	export class CreateFacultyRequest {
	    name: string;
	
	    static createFrom(source: any = {}) {
	        return new CreateFacultyRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	    }
	}
	export class CreateLessonRequest {
	    semester_id: number;
	    day_of_week: number;
	    start_time: string;
	    end_time: string;
	    direction_id: number;
	    teacher_count?: number;
	    faculty_ids: number[];
	    teacher_ids: number[];
	
	    static createFrom(source: any = {}) {
	        return new CreateLessonRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.semester_id = source["semester_id"];
	        this.day_of_week = source["day_of_week"];
	        this.start_time = source["start_time"];
	        this.end_time = source["end_time"];
	        this.direction_id = source["direction_id"];
	        this.teacher_count = source["teacher_count"];
	        this.faculty_ids = source["faculty_ids"];
	        this.teacher_ids = source["teacher_ids"];
	    }
	}
	export class CreateSemesterRequest {
	    name: string;
	    start_date: string;
	    end_date: string;
	
	    static createFrom(source: any = {}) {
	        return new CreateSemesterRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.start_date = source["start_date"];
	        this.end_date = source["end_date"];
	    }
	}
	export class CreateTeacherRequest {
	    first_name: string;
	    last_name: string;
	    middle_name: string;
	    direction_id: number;
	    rate: number;
	
	    static createFrom(source: any = {}) {
	        return new CreateTeacherRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.first_name = source["first_name"];
	        this.last_name = source["last_name"];
	        this.middle_name = source["middle_name"];
	        this.direction_id = source["direction_id"];
	        this.rate = source["rate"];
	    }
	}
	export class Direction {
	    id: number;
	    name: string;
	    is_archived: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Direction(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.is_archived = source["is_archived"];
	    }
	}
	export class Faculty {
	    id: number;
	    name: string;
	    is_archived: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Faculty(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.is_archived = source["is_archived"];
	    }
	}
	export class Lesson {
	    id: number;
	    semester_id: number;
	    day_of_week: number;
	    start_time: string;
	    end_time: string;
	    direction_id: number;
	    teacher_count?: number;
	    semester_name?: string;
	    direction_name?: string;
	    faculty_names?: string[];
	    teacher_names?: string[];
	    faculty_ids?: number[];
	    teacher_ids?: number[];
	
	    static createFrom(source: any = {}) {
	        return new Lesson(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.semester_id = source["semester_id"];
	        this.day_of_week = source["day_of_week"];
	        this.start_time = source["start_time"];
	        this.end_time = source["end_time"];
	        this.direction_id = source["direction_id"];
	        this.teacher_count = source["teacher_count"];
	        this.semester_name = source["semester_name"];
	        this.direction_name = source["direction_name"];
	        this.faculty_names = source["faculty_names"];
	        this.teacher_names = source["teacher_names"];
	        this.faculty_ids = source["faculty_ids"];
	        this.teacher_ids = source["teacher_ids"];
	    }
	}
	export class Semester {
	    id: number;
	    name: string;
	    start_date: string;
	    end_date: string;
	    is_archived: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Semester(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.start_date = source["start_date"];
	        this.end_date = source["end_date"];
	        this.is_archived = source["is_archived"];
	    }
	}
	export class Teacher {
	    id: number;
	    first_name: string;
	    last_name: string;
	    middle_name: string;
	    direction_id: number;
	    rate: number;
	    direction_name?: string;
	    is_archived: boolean;
	    is_guest: boolean;
	    is_bound?: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Teacher(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.first_name = source["first_name"];
	        this.last_name = source["last_name"];
	        this.middle_name = source["middle_name"];
	        this.direction_id = source["direction_id"];
	        this.rate = source["rate"];
	        this.direction_name = source["direction_name"];
	        this.is_archived = source["is_archived"];
	        this.is_guest = source["is_guest"];
	        this.is_bound = source["is_bound"];
	    }
	}
	export class UnbindTeacherFromSemesterRequest {
	    semester_id: number;
	    teacher_id: number;
	
	    static createFrom(source: any = {}) {
	        return new UnbindTeacherFromSemesterRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.semester_id = source["semester_id"];
	        this.teacher_id = source["teacher_id"];
	    }
	}
	export class UpdateDirectionRequest {
	    id: number;
	    name: string;
	
	    static createFrom(source: any = {}) {
	        return new UpdateDirectionRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	    }
	}
	export class UpdateFacultyRequest {
	    id: number;
	    name: string;
	
	    static createFrom(source: any = {}) {
	        return new UpdateFacultyRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	    }
	}
	export class UpdateLessonRequest {
	    id: number;
	    semester_id: number;
	    day_of_week: number;
	    start_time: string;
	    end_time: string;
	    direction_id: number;
	    teacher_count?: number;
	    faculty_ids: number[];
	    teacher_ids: number[];
	
	    static createFrom(source: any = {}) {
	        return new UpdateLessonRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.semester_id = source["semester_id"];
	        this.day_of_week = source["day_of_week"];
	        this.start_time = source["start_time"];
	        this.end_time = source["end_time"];
	        this.direction_id = source["direction_id"];
	        this.teacher_count = source["teacher_count"];
	        this.faculty_ids = source["faculty_ids"];
	        this.teacher_ids = source["teacher_ids"];
	    }
	}
	export class UpdateSemesterRequest {
	    id: number;
	    name: string;
	    start_date: string;
	    end_date: string;
	
	    static createFrom(source: any = {}) {
	        return new UpdateSemesterRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.start_date = source["start_date"];
	        this.end_date = source["end_date"];
	    }
	}
	export class UpdateTeacherRequest {
	    id: number;
	    first_name: string;
	    last_name: string;
	    middle_name: string;
	    direction_id: number;
	    rate: number;
	
	    static createFrom(source: any = {}) {
	        return new UpdateTeacherRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.first_name = source["first_name"];
	        this.last_name = source["last_name"];
	        this.middle_name = source["middle_name"];
	        this.direction_id = source["direction_id"];
	        this.rate = source["rate"];
	    }
	}

}

