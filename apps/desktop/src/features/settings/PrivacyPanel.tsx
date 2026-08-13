export function PrivacyPanel() {
  return (
    <section className="privacy-panel">
      <span className="privacy-shield">◇</span>
      <div>
        <h2>Riêng tư theo mặc định</h2>
        <p>
          Ảnh màn hình, âm thanh, transcript và văn bản đọc chính tả chỉ nằm
          trong bộ nhớ cho phiên hiện tại. Tro không ghi màn hình nền và không
          lưu lịch sử nội dung.
        </p>
        <small>
          Máy chủ chỉ giữ bộ đếm sử dụng, mã lỗi ổn định và quyết định allow /
          confirm / block.
        </small>
      </div>
    </section>
  );
}
