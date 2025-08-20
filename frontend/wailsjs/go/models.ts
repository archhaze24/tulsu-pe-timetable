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
	
	    static createFrom(source: any = {}) {
	        return new Config(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.dbPath = source["dbPath"];
	    }
	}

}

export namespace storage {
	
	export class CreateDirectionRequest {
	    name: string;
	    description: string;
	
	    static createFrom(source: any = {}) {
	        return new CreateDirectionRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.description = source["description"];
	    }
	}
	export class CreateFacultyRequest {
	    name: string;
	    short_name: string;
	
	    static createFrom(source: any = {}) {
	        return new CreateFacultyRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.short_name = source["short_name"];
	    }
	}
	export class CreateLessonRequest {
	    faculty_id: number;
	    direction_id: number;
	    teacher_id?: number;
	    day_of_week: number;
	    lesson_number: number;
	
	    static createFrom(source: any = {}) {
	        return new CreateLessonRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.faculty_id = source["faculty_id"];
	        this.direction_id = source["direction_id"];
	        this.teacher_id = source["teacher_id"];
	        this.day_of_week = source["day_of_week"];
	        this.lesson_number = source["lesson_number"];
	    }
	}
	export class CreateTeacherRequest {
	    last_name: string;
	    first_name: string;
	    middle_name: string;
	    rate: number;
	
	    static createFrom(source: any = {}) {
	        return new CreateTeacherRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.last_name = source["last_name"];
	        this.first_name = source["first_name"];
	        this.middle_name = source["middle_name"];
	        this.rate = source["rate"];
	    }
	}
	export class Direction {
	    id: number;
	    name: string;
	    description: string;
	    // Go type: time
	    created_at: any;
	    // Go type: time
	    updated_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Direction(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.description = source["description"];
	        this.created_at = this.convertValues(source["created_at"], null);
	        this.updated_at = this.convertValues(source["updated_at"], null);
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
	export class Faculty {
	    id: number;
	    name: string;
	    short_name: string;
	    // Go type: time
	    created_at: any;
	    // Go type: time
	    updated_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Faculty(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.short_name = source["short_name"];
	        this.created_at = this.convertValues(source["created_at"], null);
	        this.updated_at = this.convertValues(source["updated_at"], null);
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
	export class Lesson {
	    id: number;
	    faculty_id: number;
	    direction_id: number;
	    teacher_id?: number;
	    day_of_week: number;
	    lesson_number: number;
	    // Go type: time
	    created_at: any;
	    // Go type: time
	    updated_at: any;
	    faculty_name?: string;
	    direction_name?: string;
	    teacher_name?: string;
	
	    static createFrom(source: any = {}) {
	        return new Lesson(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.faculty_id = source["faculty_id"];
	        this.direction_id = source["direction_id"];
	        this.teacher_id = source["teacher_id"];
	        this.day_of_week = source["day_of_week"];
	        this.lesson_number = source["lesson_number"];
	        this.created_at = this.convertValues(source["created_at"], null);
	        this.updated_at = this.convertValues(source["updated_at"], null);
	        this.faculty_name = source["faculty_name"];
	        this.direction_name = source["direction_name"];
	        this.teacher_name = source["teacher_name"];
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
	export class Teacher {
	    id: number;
	    last_name: string;
	    first_name: string;
	    middle_name: string;
	    rate: number;
	    // Go type: time
	    created_at: any;
	    // Go type: time
	    updated_at: any;
	
	    static createFrom(source: any = {}) {
	        return new Teacher(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.last_name = source["last_name"];
	        this.first_name = source["first_name"];
	        this.middle_name = source["middle_name"];
	        this.rate = source["rate"];
	        this.created_at = this.convertValues(source["created_at"], null);
	        this.updated_at = this.convertValues(source["updated_at"], null);
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
	export class UpdateDirectionRequest {
	    id: number;
	    name: string;
	    description: string;
	
	    static createFrom(source: any = {}) {
	        return new UpdateDirectionRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.description = source["description"];
	    }
	}
	export class UpdateFacultyRequest {
	    id: number;
	    name: string;
	    short_name: string;
	
	    static createFrom(source: any = {}) {
	        return new UpdateFacultyRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.short_name = source["short_name"];
	    }
	}
	export class UpdateLessonRequest {
	    id: number;
	    faculty_id: number;
	    direction_id: number;
	    teacher_id?: number;
	    day_of_week: number;
	    lesson_number: number;
	
	    static createFrom(source: any = {}) {
	        return new UpdateLessonRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.faculty_id = source["faculty_id"];
	        this.direction_id = source["direction_id"];
	        this.teacher_id = source["teacher_id"];
	        this.day_of_week = source["day_of_week"];
	        this.lesson_number = source["lesson_number"];
	    }
	}
	export class UpdateTeacherRequest {
	    id: number;
	    last_name: string;
	    first_name: string;
	    middle_name: string;
	    rate: number;
	
	    static createFrom(source: any = {}) {
	        return new UpdateTeacherRequest(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.last_name = source["last_name"];
	        this.first_name = source["first_name"];
	        this.middle_name = source["middle_name"];
	        this.rate = source["rate"];
	    }
	}

}

