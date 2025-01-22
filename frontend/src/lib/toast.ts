export enum ToastType {
	Success,
	Error
}

export class Toast {
	public id: string;
	public message: string;
	public type: ToastType;

	constructor(id: string, message: string, type: ToastType) {
		this.id = id;
		this.message = message;
		this.type = type;
	}

	static success(message: string): Toast {
		return new Toast(Date.now().toString(), message, ToastType.Success);
	}

	static error(message: string): Toast {
		return new Toast(Date.now().toString(), message, ToastType.Error);
	}
}
