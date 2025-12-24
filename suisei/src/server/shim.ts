import Buffer from "buffer";

(globalThis as any).Buffer = Buffer;

(globalThis as any).Intl = {
  DateTimeFormat: class {
    constructor(
      locales?: string | string[] | undefined,
      options?: Intl.DateTimeFormatOptions | undefined
    ) {}
    format(date?: Date | number): string {
      return new Date(date || Date.now()).toISOString();
    }
    static supportedLocalesOf(
      locales: string | string[],
      options?: Intl.DateTimeFormatOptions
    ): string[] {
      throw new Error("Not implemented");
    }
    formatToParts(date?: Date | number): Intl.DateTimeFormatPart[] {
      throw new Error("Not implemented");
    }
    resolvedOptions(): Intl.ResolvedDateTimeFormatOptions {
      throw new Error("Not implemented");
    }
  } as any,
  // NumberFormat: class {
  //   constructor(locales, options) {}
  //   format(number) {
  //     return String(number);
  //   }
  //   resolvedOptions() {
  //     return { locale: "en-US" };
  //   }
  // },
  // Segmenter: class {
  //   segment(input) {
  //     return input.split("");
  //   }
  // },
} as any;
