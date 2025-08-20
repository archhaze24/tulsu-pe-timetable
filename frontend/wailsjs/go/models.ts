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

