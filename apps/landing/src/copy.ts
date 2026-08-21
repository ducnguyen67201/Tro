export type DemoPlaybackState = "ready" | "playing" | "paused" | "ended";

export type Locale = "vi" | "en";

type Feature = {
  number: string;
  title: string;
  body: string;
};

export type SiteCopy = {
  meta: {
    title: string;
    description: string;
  };
  language: {
    label: string;
    vietnamese: string;
    english: string;
  };
  header: {
    homeLabel: string;
    navigationLabel: string;
    howItWorks: string;
    whyTro: string;
    backToTop: string;
    systemStatus: string;
    getTro: string;
  };
  hero: {
    practiceWindow: string;
    topic: string;
    explanationWindow: string;
    previewSteps: [string, string, string];
    voiceWindow: string;
    listening: string;
    voicePrompt: string;
    notesFolder: string;
    progressFolder: string;
    codeVariable: string;
    codeValue: string;
    tagline: string;
    description: string;
    primaryCta: string;
    secondaryCta: string;
    shortcutPrefix: string;
    shortcutSuffix: string;
    noteWindow: string;
    noteKicker: string;
    noteBody: string;
    noteAria: string;
  };
  partners: {
    label: string;
    title: string;
    intro: string;
    entries: Array<{
      featuredLabel: string;
      name: string;
      description: string;
      visit: string;
      linkLabel: string;
      website: string;
      logo: string;
    }>;
  };
  demo: {
    label: string;
    title: string;
    statuses: Record<DemoPlaybackState, string>;
    play: string;
    playLabel: string;
    replay: string;
    replayLabel: string;
    footageLabel: string;
    duration: string;
    videoLabel: string;
    fallback: string;
    chaptersLabel: string;
    chapters: [string, string];
  };
  principles: {
    label: string;
    title: string;
    features: [Feature, Feature, Feature];
  };
  download: {
    label: string;
    title: string;
    body: string;
    versionLabel: string;
    version: string;
    platformLabel: string;
    platform: string;
    sizeLabel: string;
    size: string;
    allPlatformsAvailable: string;
    previewPlatformsAvailable: string;
    previewStatus: string;
    unsignedPreviewStatus: string;
    platformsLabel: string;
    platforms: {
      macosApple: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
      macosIntel: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
      windows: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
    };
    accessNote: string;
    previewNote: string;
    unsignedPreviewWarning: string;
    signingDisclosure: string;
    codeSigningPolicy: string;
    privacyPolicy: string;
  };
  closing: {
    codeObject: string;
    firstLine: string;
    secondLine: string;
    cta: string;
  };
  footer: {
    statement: string;
  };
};

export const siteCopy: Record<Locale, SiteCopy> = {
  vi: {
    meta: {
      title: "Tro — Học nhẹ hơn. Tiến xa hơn.",
      description:
        "Tro là người bạn học ưu tiên tiếng Việt: hiểu màn hình, lắng nghe câu hỏi và hướng dẫn bạn từng bước.",
    },
    language: {
      label: "Chọn ngôn ngữ",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Trang chủ Tro",
      navigationLabel: "Điều hướng chính",
      howItWorks: "cách hoạt động",
      whyTro: "vì sao chọn tro",
      backToTop: "Về đầu trang",
      systemStatus: "Trạng thái hệ thống Tro",
      getTro: "tải tro",
    },
    hero: {
      practiceWindow: "bài_tập_04",
      topic: "HÀM BẬC HAI",
      explanationWindow: "tro · lời giải thích",
      previewSteps: [
        "nhận ra dạng bài",
        "đối chiếu giá trị",
        "tự tin chọn đáp án",
      ],
      voiceWindow: "giọng nói",
      listening: "Đang lắng nghe",
      voicePrompt: "“Em chưa hiểu chỗ này…”",
      notesFolder: "ghi chú",
      progressFolder: "tiến độ",
      codeVariable: "học",
      codeValue: "dễ",
      tagline: "học nhẹ hơn. tiến xa hơn.",
      description:
        "Một người bạn học ưu tiên tiếng Việt: hiểu ứng dụng bạn đang mở, lắng nghe khi bạn mắc kẹt và cùng bạn đi qua từng bước tiếp theo.",
      primaryCta: "xem tro hoạt động",
      secondaryCta: "vì sao học sinh chọn tro",
      shortcutPrefix: "Nhấn",
      shortcutSuffix: "ở bất cứ đâu · macOS 13+ · Windows 10/11",
      noteWindow: "tro nhắn",
      noteKicker: "Không chỉ là một ô đáp án.",
      noteBody: "Một gia sư có mặt ngay trên màn hình của bạn.",
      noteAria: "Nguyên tắc sản phẩm của Tro",
    },
    partners: {
      label: "Đối tác đồng hành",
      title: "Cùng người học đi xa hơn.",
      intro:
        "Tro hợp tác với những nhà giáo dục tin rằng việc học nên gần gũi, rõ ràng và thực tế.",
      entries: [
        {
          featuredLabel: "Đối tác giáo dục đầu tiên",
          name: "Just Tin English",
          description:
            "Tiếng Anh cho người học lại từ đầu—từ ngữ pháp, từ vựng, nghe và nói đến TOEIC và tiếng Anh thương mại.",
          visit: "Khám phá Just Tin English",
          linkLabel: "Mở trang web Just Tin English trong tab mới",
          website: "https://www.justtinenglish.com/",
          logo: "/partners/just-tin-english.png",
        },
      ],
    },
    demo: {
      label: "Cảnh quay thật. Thao tác thật.",
      title: "Xem Tro làm việc thật.",
      statuses: {
        ready: "Video sẵn sàng",
        playing: "Đang phát demo thật",
        paused: "Video đã tạm dừng",
        ended: "Bạn đã xem xong",
      },
      play: "Phát video",
      playLabel: "Phát video demo Tro",
      replay: "Xem lại",
      replayLabel: "Phát lại video demo Tro từ đầu",
      footageLabel: "CẢNH QUAY THẬT",
      duration: "31 GIÂY",
      videoLabel:
        "Tro thao tác trực tiếp trên Scratch và Google Sheets bằng hướng dẫn tiếng Việt.",
      fallback: "Trình duyệt của bạn không hỗ trợ video này.",
      chaptersLabel: "Nội dung video",
      chapters: ["01 · Scratch", "02 · Google Sheets"],
    },
    principles: {
      label: "Dành cho khoảnh khắc bạn suýt bỏ cuộc.",
      title: "Từ “em bị kẹt” đến “em làm được rồi”.",
      features: [
        {
          number: "01",
          title: "Hiểu màn hình đang mở",
          body: "Khi tác vụ cần dùng màn hình, Tro quan sát ứng dụng đang hoạt động trước, xin phép trước thay đổi quan trọng và hiển thị viền màu trong lúc điều khiển.",
        },
        {
          number: "02",
          title: "Lắng nghe tự nhiên",
          body: "Hỏi bằng tiếng Việt, tiếng Anh hoặc cả hai—như khi bạn hỏi một người bạn cùng lớp.",
        },
        {
          number: "03",
          title: "Hướng dẫn rồi lùi lại",
          body: "Nhận một lộ trình rõ ràng qua bài toán mà vẫn giữ trọn khoảnh khắc tự mình hiểu ra.",
        },
      ],
    },
    download: {
      label: "Ứng dụng máy tính",
      title: "Tro dành cho Mac và Windows.",
      body: "Chọn đúng phiên bản cho máy của bạn để nhận hướng dẫn bằng giọng nói và ngay trên màn hình khi đang học.",
      versionLabel: "Phiên bản",
      version: "0.1.7",
      platformLabel: "Hệ điều hành",
      platform: "macOS + Windows",
      sizeLabel: "Tình trạng",
      size: "Đang tải thông tin bản phát hành",
      allPlatformsAvailable: "Mac Apple silicon, Mac Intel và Windows có sẵn",
      previewPlatformsAvailable: "Có bản xem trước chưa ký cho máy tính",
      previewStatus: "Bản xem trước",
      unsignedPreviewStatus: "Bản xem trước chưa ký",
      platformsLabel: "Chọn phiên bản Tro cho máy tính",
      platforms: {
        macosApple: {
          badge: "MAC",
          name: "macOS · Apple",
          requirements: "macOS 13+ · Apple silicon",
          size: "139 MB ZIP",
          status: "Sắp ra mắt",
          cta: "Mac sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Mac",
        },
        macosIntel: {
          badge: "MAC",
          name: "macOS · Intel",
          requirements: "macOS 13+ · Intel",
          size: "Bản x64",
          status: "Sắp ra mắt",
          cta: "Mac Intel sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Mac Intel",
        },
        windows: {
          badge: "WIN",
          name: "Windows",
          requirements: "Windows 10/11 · x64",
          size: "Bản x64",
          status: "Sắp ra mắt",
          cta: "Windows sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Windows",
        },
      },
      accessNote: "Đăng nhập bằng Google để bắt đầu sử dụng Tro.",
      previewNote:
        "Các nút tải tự động trỏ tới bản phát hành Tro mới nhất trên GitHub.",
      unsignedPreviewWarning:
        "Các bản Mac và Windows hiện là bản xem trước chưa ký. macOS Gatekeeper hoặc Windows SmartScreen có thể hiển thị cảnh báo trước khi cài đặt.",
      signingDisclosure:
        "Bản Mac ổn định dùng Apple Developer ID và notarization; bản Windows ổn định dùng SignPath.io, chứng thư bởi SignPath Foundation.",
      codeSigningPolicy: "Chính sách ký mã (Code signing policy)",
      privacyPolicy: "Chính sách quyền riêng tư",
    },
    closing: {
      codeObject: "hocSinh.tuTin",
      firstLine: "Bớt mắc kẹt.",
      secondLine: "Giỏi lên mỗi ngày.",
      cta: "Trải nghiệm cách Tro giúp",
    },
    footer: {
      statement: "Ưu tiên tiếng Việt. Luôn đặt người học trước.",
    },
  },
  en: {
    meta: {
      title: "Tro — Study easier. Become your best.",
      description:
        "Tro is the Vietnamese-first desktop tutor that understands the app you have open, listens to your question, and guides the next step.",
    },
    language: {
      label: "Choose language",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Tro home",
      navigationLabel: "Main navigation",
      howItWorks: "how it works",
      whyTro: "why tro",
      backToTop: "Back to top",
      systemStatus: "Tro system status",
      getTro: "download",
    },
    hero: {
      practiceWindow: "practice_04",
      topic: "QUADRATIC FUNCTIONS",
      explanationWindow: "tro · explanation",
      previewSteps: [
        "spot the pattern",
        "match the values",
        "choose confidently",
      ],
      voiceWindow: "voice",
      listening: "Listening",
      voicePrompt: "“I don’t understand this part…”",
      notesFolder: "notes",
      progressFolder: "progress",
      codeVariable: "study",
      codeValue: "easy",
      tagline: "study easy. become your best.",
      description:
        "A Vietnamese-first study buddy that understands the app you have open, listens when you are stuck, and walks you through the next step.",
      primaryCta: "see tro work",
      secondaryCta: "why students use tro",
      shortcutPrefix: "Press",
      shortcutSuffix: "anywhere · macOS 13+ · Windows 10/11",
      noteWindow: "tro says",
      noteKicker: "Not another answer box.",
      noteBody: "A tutor that meets you on the screen.",
      noteAria: "Tro product principle",
    },
    partners: {
      label: "Learning partner",
      title: "Better learning, built together.",
      intro:
        "Tro partners with educators who believe learning should feel approachable, clear, and practical.",
      entries: [
        {
          featuredLabel: "Our first education partner",
          name: "Just Tin English",
          description:
            "English for learners starting again—from grammar, vocabulary, listening, and speaking to TOEIC and business English.",
          visit: "Explore Just Tin English",
          linkLabel: "Open the Just Tin English website in a new tab",
          website: "https://www.justtinenglish.com/",
          logo: "/partners/just-tin-english.png",
        },
      ],
    },
    demo: {
      label: "Real footage. Real actions.",
      title: "See Tro work for real.",
      statuses: {
        ready: "Video ready",
        playing: "Playing the real demo",
        paused: "Video paused",
        ended: "Demo complete",
      },
      play: "Play video",
      playLabel: "Play the Tro demo video",
      replay: "Replay",
      replayLabel: "Replay the Tro demo video from the beginning",
      footageLabel: "REAL APP FOOTAGE",
      duration: "31 SECONDS",
      videoLabel:
        "Tro works directly in Scratch and Google Sheets with Vietnamese guidance.",
      fallback: "Your browser does not support this video.",
      chaptersLabel: "Video chapters",
      chapters: ["01 · Scratch", "02 · Google Sheets"],
    },
    principles: {
      label: "Made for the moment you almost give up.",
      title: "From “I’m stuck” to “I’ve got this.”",
      features: [
        {
          number: "01",
          title: "Understands the open screen",
          body: "When a task needs the screen, Tro observes the active app first, asks before consequential changes, and shows a colored border while it is in control.",
        },
        {
          number: "02",
          title: "Listens naturally",
          body: "Ask in Vietnamese, English, or both—just like you would ask a classmate.",
        },
        {
          number: "03",
          title: "Guides, then steps back",
          body: "Get a clear path through the problem without losing the learning moment.",
        },
      ],
    },
    download: {
      label: "Desktop app",
      title: "Tro for Mac and Windows.",
      body: "Choose the right build for your computer to get voice and on-screen guidance while you study.",
      versionLabel: "Version",
      version: "0.1.7",
      platformLabel: "Systems",
      platform: "macOS + Windows",
      sizeLabel: "Availability",
      size: "Loading release information",
      allPlatformsAvailable:
        "Apple silicon Mac, Intel Mac, and Windows available",
      previewPlatformsAvailable: "Unsigned desktop previews available",
      previewStatus: "Preview",
      unsignedPreviewStatus: "Unsigned preview",
      platformsLabel: "Choose a Tro desktop version",
      platforms: {
        macosApple: {
          badge: "MAC",
          name: "macOS · Apple",
          requirements: "macOS 13+ · Apple silicon",
          size: "139 MB ZIP",
          status: "Coming soon",
          cta: "Mac coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Mac",
        },
        macosIntel: {
          badge: "MAC",
          name: "macOS · Intel",
          requirements: "macOS 13+ · Intel",
          size: "x64 build",
          status: "Coming soon",
          cta: "Intel Mac coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Intel Mac",
        },
        windows: {
          badge: "WIN",
          name: "Windows",
          requirements: "Windows 10/11 · x64",
          size: "x64 build",
          status: "Coming soon",
          cta: "Windows coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Windows",
        },
      },
      accessNote: "Sign in with Google to start using Tro.",
      previewNote:
        "Download buttons automatically follow the latest Tro release on GitHub.",
      unsignedPreviewWarning:
        "The current Mac and Windows builds are unsigned previews. macOS Gatekeeper or Windows SmartScreen may warn before installation.",
      signingDisclosure:
        "Stable Mac builds use Apple Developer ID and notarization; stable Windows builds use SignPath.io, certificate by SignPath Foundation.",
      codeSigningPolicy: "Code signing policy",
      privacyPolicy: "Privacy policy",
    },
    closing: {
      codeObject: "student.confidence",
      firstLine: "Less stuck.",
      secondLine: "More becoming.",
      cta: "Experience the flow",
    },
    footer: {
      statement: "Vietnamese-first. Student-always.",
    },
  },
};
